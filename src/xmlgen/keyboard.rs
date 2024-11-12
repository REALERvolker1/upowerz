//! # D-Bus interface proxy for: `org.freedesktop.UPower.KbdBacklight`
//!
//! This code was generated by `zbus-xmlgen` `4.1.0` from D-Bus introspection data.
//! Source: `Interface '/org/freedesktop/UPower/KbdBacklight' from service 'org.freedesktop.UPower' on system bus`.
//!
//! You may prefer to adapt it, instead of using it verbatim.
//!
//! More information can be found in the [Writing a client proxy] section of the zbus
//! documentation.
//!
//! This type implements the [D-Bus standard interfaces], (`org.freedesktop.DBus.*`) for which the
//! following zbus API can be used:
//!
//! * [`zbus::fdo::PropertiesProxy`]
//! * [`zbus::fdo::IntrospectableProxy`]
//! * [`zbus::fdo::PeerProxy`]
//!
//! Consequently `zbus-xmlgen` did not generate code for the above interfaces.
//!
//! [Writing a client proxy]: https://dbus2.github.io/zbus/client.html
//! [D-Bus standard interfaces]: https://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces,

use zbus::proxy;
#[proxy(
    interface = "org.freedesktop.UPower.KbdBacklight",
    default_service = "org.freedesktop.UPower",
    default_path = "/org/freedesktop/UPower/KbdBacklight",
    gen_blocking = false
)]
pub trait KbdBacklight {
    /// GetBrightness method
    fn get_brightness(&self) -> zbus::Result<i32>;

    /// GetMaxBrightness method
    fn get_max_brightness(&self) -> zbus::Result<i32>;

    /// SetBrightness method
    fn set_brightness(&self, value: i32) -> zbus::Result<()>;

    /// BrightnessChanged signal
    #[zbus(signal)]
    fn brightness_changed(&self, value: i32) -> zbus::Result<()>;

    /// BrightnessChangedWithSource signal
    #[zbus(signal)]
    fn brightness_changed_with_source(&self, value: i32, source: &str) -> zbus::Result<()>;
}
