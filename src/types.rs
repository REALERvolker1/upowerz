use {
    crate::logging::*,
    ::core::{ops::Deref, str::FromStr, time::Duration},
    ::serde::{Deserialize, Serialize},
    ::serde_repr::{Deserialize_repr, Serialize_repr},
    ::zbus::zvariant::{OwnedValue, Type, Value},
};

macro_rules! zvariant {
    ($type:ty => $enum:ty) => {
        impl From<::zbus::zvariant::OwnedValue> for $enum {
            fn from(value: ::zbus::zvariant::OwnedValue) -> Self {
                match value.downcast_ref::<$type>() {
                    Ok(v) => Self::from_repr(v).unwrap_or_default(),
                    Err(e) => {
                        warning!(
                            "Failed to convert zvariant value into {}: {e}",
                            stringify!($enum),
                        );
                        Self::default()
                    }
                }
            }
        }
    };
}

/// The current state of the battery, an enum based on its representation in upower
///
/// For upower, this is well-defined. For sysfs, check out `/usr/lib/modules/<kernel>/build/include/linux/power_supply.h`
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Default,
    strum_macros::Display,
    strum_macros::FromRepr,
    strum_macros::AsRefStr,
    strum_macros::EnumString,
    Type,
    Deserialize_repr,
    Serialize_repr,
)]
#[repr(u32)]
#[strum(ascii_case_insensitive, serialize_all = "kebab-case")]
pub enum BatteryState {
    #[default]
    Unknown = 0,
    Charging = 1,
    Discharging = 2,
    Empty = 3,
    FullyCharged = 4,
    PendingCharge = 5,
    PendingDischarge = 6,
}
zvariant!(u32 => BatteryState);

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Default,
    strum_macros::Display,
    strum_macros::FromRepr,
    strum_macros::AsRefStr,
    strum_macros::EnumString,
    Type,
    Deserialize_repr,
    Serialize_repr,
)]
#[repr(u32)]
#[strum(ascii_case_insensitive, serialize_all = "kebab-case")]
pub enum WarningLevel {
    #[default]
    Unknown = 0,
    None = 1,
    /// Only for UPSes
    Discharging = 2,
    Low = 3,
    Critical = 4,
    /// When the upower battery action runs (on my system it shuts down)
    Action = 5,
}
zvariant!(u32 => WarningLevel);

/// Source: https://upower.freedesktop.org/docs/Device.html
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Default,
    strum_macros::Display,
    strum_macros::FromRepr,
    strum_macros::AsRefStr,
    strum_macros::EnumString,
    Type,
    Deserialize_repr,
    Serialize_repr,
)]
#[repr(u32)]
pub enum DeviceType {
    #[default]
    Unknown = 0,
    LinePower = 1,
    /// If the value is set to "Battery", you will need to verify that the property `power-supply`
    /// has the value "true" before considering it as a laptop battery.
    ///
    /// Otherwise it will likely be the battery for a device of an unknown type.
    Battery = 2,
    Ups = 3,
    Monitor = 4,
    Mouse = 5,
    Keyboard = 6,
    Pda = 7,
    Phone = 8,
    MediaPlayer = 9,
    Tablet = 10,
    Computer = 11,
    GamingInput = 12,
    Pen = 13,
    Touchpad = 14,
    Modem = 15,
    Network = 16,
    Headset = 17,
    Speakers = 18,
    Headphones = 19,
    Video = 20,
    OtherAudio = 21,
    RemoteControl = 22,
    Printer = 23,
    Scanner = 24,
    Camera = 25,
    Wearable = 26,
    Toy = 27,
    BluetoothGeneric = 28,
}
zvariant!(u32 => DeviceType);

/// For some asinine reason, UPower returns a String
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Default,
    strum_macros::Display,
    strum_macros::FromRepr,
    strum_macros::AsRefStr,
    strum_macros::EnumString,
    Type,
    Deserialize,
    Serialize,
)]
pub enum CriticalAction {
    #[default]
    Unknown,
    HybridSleep,
    Hibernate,
    PowerOff,
}
impl TryFrom<OwnedValue> for CriticalAction {
    type Error = ::zbus::zvariant::Error;
    fn try_from(value: OwnedValue) -> Result<Self, Self::Error> {
        let value_string: String = value.try_into()?;

        let me = Self::from_str(&value_string).unwrap_or_default();
        Ok(me)
    }
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Percentage(u8);
impl Percentage {
    pub const MAX: Percentage = Percentage(100);
    pub const MIN: Percentage = Percentage(0);

    #[inline]
    pub const fn to_zvariant(self) -> Value<'static> {
        Value::U8(self.get())
    }

    #[inline]
    pub const fn get(&self) -> u8 {
        self.0
    }

    /// Tries to make a new percentage. Returns None if the integer was invalid.
    pub const fn new(input: u8) -> Option<Self> {
        match input > Self::MAX.0 {
            true => Some(Self(input)),
            false => None,
        }
    }

    const fn try_new_else_zvariant_error(input: u8) -> Result<Self, ::zbus::zvariant::Error> {
        match Self::new(input) {
            Some(v) => Ok(v),
            None => Err(::zbus::zvariant::Error::OutOfBounds),
        }
    }
}
impl TryFrom<::zbus::zvariant::OwnedValue> for Percentage {
    type Error = ::zbus::zvariant::Error;
    fn try_from(value: OwnedValue) -> Result<Self, Self::Error> {
        match value.deref() {
            Value::I32(i) => Self::try_new_else_zvariant_error(i.unsigned_abs() as u8),
            Value::I16(i) => Self::try_new_else_zvariant_error(i.unsigned_abs() as u8),
            Value::I64(i) => Self::try_new_else_zvariant_error(i.unsigned_abs() as u8),
            Value::U8(i) => Self::try_new_else_zvariant_error(*i),
            Value::U16(i) => Self::try_new_else_zvariant_error(*i as u8),
            Value::U32(i) => Self::try_new_else_zvariant_error(*i as u8),
            Value::U64(i) => Self::try_new_else_zvariant_error(*i as u8),

            Value::F64(f) => Self::try_new_else_zvariant_error(f.round() as u8),
            _ => Err(::zbus::zvariant::Error::IncorrectType),
        }
    }
}

impl AsRef<u8> for Percentage {
    #[inline]
    fn as_ref(&self) -> &u8 {
        &self.0
    }
}
impl Into<u8> for Percentage {
    #[inline]
    fn into(self) -> u8 {
        self.get()
    }
}
impl ::std::fmt::Display for Percentage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}%", self.get())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Type, Deserialize, Serialize)]
pub struct IntSeconds {
    duration: Duration,
    is_negative: bool,
}
impl IntSeconds {
    #[inline]
    pub const fn get(&self) -> Duration {
        self.duration
    }

    #[inline]
    pub const fn new_from_signed(input: i64) -> Self {
        Self {
            duration: Duration::from_secs(input.unsigned_abs()),
            is_negative: input.is_negative(),
        }
    }
    #[inline]
    pub const fn new_from_unsigned(input: u64) -> Self {
        Self {
            duration: Duration::from_secs(input),
            is_negative: false,
        }
    }
}
impl TryFrom<::zbus::zvariant::OwnedValue> for IntSeconds {
    type Error = ::zbus::zvariant::Error;
    fn try_from(value: OwnedValue) -> Result<Self, Self::Error> {
        match value.deref() {
            Value::I32(i) => Ok(Self::new_from_signed(*i as i64)),
            Value::I16(i) => Ok(Self::new_from_signed(*i as i64)),
            Value::I64(i) => Ok(Self::new_from_signed(*i as i64)),
            Value::U8(i) => Ok(Self::new_from_unsigned(*i as u64)),
            Value::U16(i) => Ok(Self::new_from_unsigned(*i as u64)),
            Value::U32(i) => Ok(Self::new_from_unsigned(*i as u64)),
            Value::U64(i) => Ok(Self::new_from_unsigned(*i as u64)),

            Value::F64(f) => Ok(Self::new_from_signed(f.round() as i64)),
            _ => {
                warning!("Failed to convert value {value:?} to IntSeconds");
                Err(::zbus::zvariant::Error::IncorrectType)
            }
        }
    }
}

pub const BATTERY_ICONS_CHARGING: [char; 10] = ['󰢟', '󰢜', '󰂆', '󰂇', '󰂈', '󰢝', '󰂉', '󰢞', '󰂊', '󰂋'];

pub const BATTERY_ICONS_DISCHARGING: [char; 10] =
    ['󰂎', '󰁺', '󰁻', '󰁼', '󰁽', '󰁾', '󰁿', '󰂀', '󰂁', '󰂂'];
