use {
    crate::{
        modules::upower::{
            types::*,
            xmlgen::{display_device::DeviceProxy, upower::UPowerProxy},
            ZbusModuleError,
        },
        preludes::*,
    },
    ::core::{future::Future, marker::PhantomData, time::Duration},
    ::zbus::{proxy::PropertyStream, Connection},
};

/*
let device_path = flags.1.get_display_device().await?;
let proxy = DeviceProxy::builder(flags.0)
            .cache_properties(::zbus::proxy::CacheProperties::No)
            .path(device_path)?
            .build()
            .await?;
*/

pub struct UPowerModule<'p, T, R>
where
    T: Sized + Send + 'static + Unpin,
    S: Stream<Item = T> + 'p + std::marker::Unpin,
    R: DeviceRequestType<T> + 'p,
{
    proxy: &'p DeviceProxy<'p>,
    listener: S,
    _type: PhantomData<R>,
    // changed_stream: PropertyStream<'p, T>,
}

impl<'p, T, R> Entity for UPowerModule<'p, T, R>
where
    T: Sized + Send + 'static + Unpin,
    S: Stream<Item = T> + 'p + std::marker::Unpin,
    R: DeviceRequestType<T> + 'p,
{
    type CreationError = zbus::Error;
    type Flags<'f> = &'p DeviceProxy<'p>;
    type MessageProduced = T;

    async fn new<'f>(flags: Self::Flags<'f>) -> Result<Self, Self::CreationError> {
        Ok(Self {
            proxy: flags,
            listener: R::receive_prop_changed(flags).await,
            _type: PhantomData,
        })
    }
}
impl<'p, T, R> DynamicEntity<S> for UPowerModule<'p, T, R>
where
    S: crate::preludes::Stream<Item = Self::MessageProduced> + std::marker::Unpin,
    T: Sized + Send + 'static + Unpin + core::fmt::Debug,
    PropertyStream<'p, T>: Stream,
    R: DeviceRequestType<T> + 'p,
{
    type MessageReceived = ();
    type RunError = zbus::Error;

    fn stream_ref<'s>(&'s mut self) -> &'s mut S {
        &mut self.listener
    }

    async fn recv_message(
        &mut self,
        message: Self::MessageReceived,
    ) -> Result<Option<Self::MessageProduced>, Self::RunError> {
        R::get(self.proxy).await.map(Option::Some)
    }
}

pub trait DeviceRequestType<O>: core::fmt::Debug + Default + Clone + Copy {
    async fn get(proxy: &DeviceProxy<'_>) -> zbus::Result<O>;
    async fn receive_prop_changed<'p>(
        proxy: &'p DeviceProxy<'p>,
    ) -> impl Stream<Item = O> + 'p + std::marker::Unpin;
    async fn get_next_prop(pstream: &mut PropertyStream<'_, O>) -> zbus::Result<O>;

    const PROPERTY_NAME: &'static str;
}
macro_rules! device_req_type {
    ($($label:ident, $getfn:ident => $type:ty),*) => {
        $(
            #[derive(Debug, Default, Clone, Copy)]
            pub struct $label;
            impl DeviceRequestType<$type> for $label {
                #[inline]
                async fn get(proxy: &DeviceProxy<'_>) -> zbus::Result<$type> {
                    proxy.$getfn().await
                }
                async fn receive_prop_changed<'p>(
                    proxy: &'p DeviceProxy<'p>,
                ) -> super::PropertyStream<'p, $type> {
                    ::paste::paste! {
                        proxy.[<receive_ $getfn _changed>]().await.then(|v| v.get()).filter_map(|v| match v {
                            Err(e) => {
                                warn!("Error getting upower property '{}': {}", Self::PROPERTY_NAME, e);
                                None
                            }
                            Ok(t) => Some(t),
                        })
                    }
                }
                async fn get_next_prop(pstream: &mut super::PropertyStream<'_, $type>) -> zbus::Result<$type> {
                    pstream.next().await.ok_or_else(|| zbus::Error::Failure(String::from("Failed to detect state change")))?.get().await
                }

                const PROPERTY_NAME: &'static str = stringify!($label);
            }
        )*
    };
}

pub mod device_request_types {
    use super::{DeviceProxy, DeviceRequestType};
    use futures_util::StreamExt;

    device_req_type! {
        Energy, energy => f64,
        EnergyFull, energy_full => f64,
        EnergyRate, energy_rate => f64,
        IconName, icon_name => String,
        IsPresent, is_present => bool,
        Percentage, percentage => crate::modules::upower::types::Percentage,
        State, state => crate::modules::upower::types::BatteryState,
        TimeToEmpty, time_to_empty => crate::modules::upower::types::IntSeconds,
        TimeToFull, time_to_full => crate::modules::upower::types::IntSeconds,
        Type, type_ => crate::modules::upower::types::DeviceType,
        WarningLevel, warning_level => crate::modules::upower::types::WarningLevel
    }
}
