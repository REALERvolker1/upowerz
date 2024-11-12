use {
    crate::{
        modules::{
            upower::{
                types::*,
                xmlgen::keyboard::{BrightnessChanged, BrightnessChangedStream, KbdBacklightProxy},
                ZbusModuleError,
            },
            AsStaticStr, EventMap, MouseEvent,
        },
        preludes::*,
    },
    ::core::{future::Future, marker::PhantomData, str::FromStr, task::Poll, time::Duration},
    ::enumflags2::{BitFlag, BitFlags},
    ::smallvec::SmallVec,
    ::zbus::{proxy::PropertyStream, Connection},
};

pub type KeyboardCallbackMap = crate::event_map_t!(MouseEvent, KeyboardEventMappings);

pub struct KeyboardModuleFlags {
    pub callbacks: KeyboardCallbackMap,
    // pub callbacks: EventMap<
    //     MouseEvent,
    //     KeyboardEventMappings,
    //     { MouseEvent::NUM_VARIANTS },
    //     { KeyboardEventMappings::NUM_VARIANTS },
    // >,
    pub increment_by_levels: i32,
}

crate::enum_variant_array! {
    KeyboardEventMappings: u8 {
        IncrementBrightness,
        DecrementBrightness,
        MaxBrightness,
        MinBrightness,
        Refresh,
    }
}

impl KeyboardEventMappings {
    pub async fn run(&self, module: &KeyboardModule<'_>) -> zbus::Result<()> {
        match self {
            KeyboardEventMappings::IncrementBrightness => {
                module.add_to_brightness(module.increment_by_levels).await
            }
            KeyboardEventMappings::DecrementBrightness => {
                module
                    .add_to_brightness(-(module.increment_by_levels))
                    .await
            }
            KeyboardEventMappings::MaxBrightness => {
                module.set_brightness(module.max_brightness).await
            }
            KeyboardEventMappings::MinBrightness => module.set_brightness(0).await,
            KeyboardEventMappings::Refresh => Ok(()),
        }
    }
}

pub struct KeyboardModule<'p, S>
where
    S: crate::preludes::Stream<Item = i32> + std::marker::Unpin,
{
    proxy: KbdBacklightProxy<'p>,
    stream: S,
    handlers: KeyboardCallbackMap,
    // this is here so I don't have to call that every time I update the brightness
    max_brightness: i32,
    increment_by_levels: i32,
}
impl<'p, S> KeyboardModule<'p, S>
where
    S: crate::preludes::Stream<Item = i32> + std::marker::Unpin,
{
    async fn fetch_update_listeners(
        &self,
        channel: &AsyncBiChannel<i32, MouseEvent>,
    ) -> Result<(), ZbusModuleError<i32>> {
        let brightness = self.proxy.get_brightness().await?;
        channel.send(brightness).await?;
        Ok(())
    }

    async fn set_brightness_percent(&self, percentage: u8) -> zbus::Result<()> {
        let percent = Percentage::try_new(percentage)?;
        let new_brightness = (self.max_brightness * (percent.get() as i32)) / 100;
        self.set_brightness(new_brightness).await?;
        Ok(())
        // let brightness = self.proxy.get_brightness().await?;
    }

    #[inline]
    async fn set_brightness(&self, value: i32) -> zbus::Result<()> {
        self.proxy.set_brightness(value).await
    }

    async fn add_to_brightness(&self, increment: i32) -> zbus::Result<()> {
        let (current, max) =
            tokio::try_join!(self.proxy.get_brightness(), self.proxy.get_max_brightness())?;

        let new = (current as i32 + increment).clamp(0, max as i32);
        self.set_brightness(new).await
    }

    // async fn handle_mouse_event(&self, event: MouseEvent) -> Result<(), ZbusModuleError<i32>> {}
}
impl<'p, S> Entity for KeyboardModule<'p, S>
where
    S: crate::preludes::Stream<Item = i32> + std::marker::Unpin,
{
    type CreationError = zbus::Error;
    type Flags<'f> = (&'f Connection, KeyboardModuleFlags);
    type MessageProduced = i32;

    async fn new(flags: Self::Flags<'f>) -> Result<Self, Self::CreationError> {
        let proxy = KbdBacklightProxy::builder(flags.0)
            .cache_properties(::zbus::proxy::CacheProperties::No)
            .build()
            .await?;

        let (change_stream, max_brightness) = tokio::try_join!(
            proxy.receive_brightness_changed(),
            proxy.get_max_brightness()
        )?;

        Ok(Self {
            max_brightness,
            stream: change_stream.map(|c| match c.message().body().deserialize() {
                Ok(b) => b,
                Err(e) => {
                    warn!("Error deserializing brightness event: {}", e);
                    0
                }
            }),
            increment_by_levels: flags.1.increment_by_levels,
            handlers: flags.1.callbacks,
            proxy,
        })
    }
}
impl<'p, S> DynamicEntity<S> for KeyboardModule<'p, S>
where
    S: crate::preludes::Stream<Item = i32> + std::marker::Unpin,
{
    type MessageReceived = MouseEvent;
    type RunError = ZbusModuleError<i32>;

    async fn recv_message(
        &mut self,
        message: Self::MessageReceived,
    ) -> Result<Option<Self::MessageProduced>, Self::RunError> {
        let Some(callbacks) = self.handlers.callbacks(message) else {
            return Ok(None);
        };
        for callback in callbacks {
            callback.run(&self).await?;
        }
        let new_brightness = self.proxy.get_brightness().await?;
        Ok(Some(new_brightness))
    }
    // async fn recv_message(&mut self, message: Self::MessageReceived) -> Result<(), Self::RunError> {
    //     let Some(callbacks) = self.handlers.callbacks(message) else {
    //         return Ok(());
    //     };
    //     for callback in callbacks {
    //         callback.run(&self).await?;
    //     }
    //     Ok(())
    // }

    fn stream_ref<'s>(&'s mut self) -> &'s mut S {
        &mut self.stream
    }
}
