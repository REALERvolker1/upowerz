use crate::types::{BatteryState, DeviceType, IntSeconds, Percentage, WarningLevel};

pub mod display_device;
pub mod keyboard;
pub mod upower;

macro_rules! disp_device_details {
    ($( $property:ident: $type:ty ),+$(,)?) => {
        /// All the details that can be provided by [`display_device`]
        #[derive(Debug, Default, Clone)]
        pub struct DisplayDeviceDetails {
            $( pub $property: $type, )+
        }
        #[derive(Debug, Clone)]
        pub struct DisplayDeviceDetailsResult {
            $( pub $property: ::zbus::Result<$type>, )+
        }

        impl Default for DisplayDeviceDetailsResult {
            fn default() -> Self {
                Self {
                    $( $property: Err(::zbus::Error::Variant(::zbus::zvariant::Error::IncorrectType)) ),+
                }
            }
        }

        impl DisplayDeviceDetailsResult {
            /// Try to extract the data payload, stopping at the first error.
            pub fn try_resolve(self) -> ::zbus::Result<DisplayDeviceDetails> {
                let mut me = DisplayDeviceDetails::default();

                $(
                    match self.$property {
                        Ok(v) => me.$property = v,
                        Err(e) => return Err(e),
                    }
                )+

                Ok(me)
            }
        }


        impl DisplayDeviceDetails {
            /// simply request all the properties.
            pub async fn request_all<'c>(proxy: &$crate::xmlgen::display_device::DeviceProxy<'c>) -> DisplayDeviceDetailsResult {
                let mut me = DisplayDeviceDetailsResult::default();
                let ($( $property ),+) = ::futures_util::join!( $(proxy.$property()),+ );
                $(
                    me.$property = $property;
                )+

                me
            }
        }
    };
}

disp_device_details! {
    energy: f64,
    energy_full: f64,
    energy_rate: f64,
    icon_name: String,
    is_present: bool,
    percentage: Percentage,
    state: BatteryState,
    time_to_empty: IntSeconds,
    time_to_full: IntSeconds,
    type_: DeviceType,
    warning_level: WarningLevel,
}
