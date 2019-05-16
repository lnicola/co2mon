#![doc(html_root_url = "https://docs.rs/co2mon/0.1.0-alpha.4")]
#![deny(missing_docs)]

//! A driver for the Holtek ([ZyAura ZG][ZG]) CO₂ USB monitors.
//!
//! The implementation was tested using a [TFA-Dostmann AIRCO2TROL MINI]
//! [AIRCO2TROL MINI] sensor.
//!
//! [AIRCO2TROL MINI]: https://www.tfa-dostmann.de/en/produkt/co2-monitor-airco2ntrol-mini/
//! [ZG]: http://www.zyaura.com/products/ZG_module.asp
//!
//! # Example usage
//!
//! ```no_run
//! # use co2mon::{Result, Sensor};
//! # fn main() -> Result<()> {
//! #
//! let sensor = Sensor::open_default()?;
//! let reading = sensor.read()?;
//! println!("{:?}", reading);
//! #
//! # Ok(())
//! # }
//! ```
//!
//! # Permissions
//!
//! On Linux, you need to be able to access the USB HID device. For that, you
//! can save the following `udev` rule to `/etc/udev/rules.d/60-co2mon.rules`:
//!
//! ```text
//! ACTION=="add|change", SUBSYSTEMS=="usb", ATTRS{idVendor}=="04d9", ATTRS{idProduct}=="a052", MODE:="0666"
//! ```
//!
//! Then reload the rules and trigger them:
//!
//! ```text
//! # udevadm control --reload
//! # udevadm trigger
//! ```
//!
//! Note that the `udev` rule above makes the device accessible to every local user.
//!
//! # References
//! The USB HID protocol is not documented, but was [reverse-engineered][had] [before][revspace].
//!
//! [had]: https://hackaday.io/project/5301/
//! [revspace]: https://revspace.nl/CO2MeterHacking

use hidapi::{HidApi, HidDevice};
use std::convert::TryFrom;
use std::ffi::CString;
use std::result;
use std::time::Duration;
use zg_co2;

pub use error::Error;
pub use zg_co2::Measurement;

mod error;

/// Result type for the fallible functions.
pub type Result<T> = result::Result<T, Error>;

/// Sensor driver struct.
///
/// # Example
///
/// ```no_run
/// # use co2mon::{Result, Sensor};
/// # fn main() -> Result<()> {
/// #
/// let sensor = Sensor::open_default()?;
/// let reading = sensor.read()?;
/// println!("{:?}", reading);
/// #
/// # Ok(())
/// # }
/// ```
pub struct Sensor {
    device: HidDevice,
    key: [u8; 8],
    timeout: i32,
}

impl Sensor {
    /// Opens the sensor device using the default USB Vendor ID (`0x04d9`) and Product ID (`0xa052`) values.
    ///
    /// When multiple devices are connected, the first one will be used.
    pub fn open_default() -> Result<Self> {
        OpenOptions::new().open()
    }

    fn open(options: &OpenOptions) -> Result<Self> {
        let hidapi = HidApi::new()?;

        const VID: u16 = 0x04d9;
        const PID: u16 = 0xa052;

        let device = match options.path_type {
            DevicePathType::Id => hidapi.open(VID, PID),
            DevicePathType::Serial(ref sn) => hidapi.open_serial(VID, PID, &sn),
            DevicePathType::Path(ref path) => hidapi.open_path(&path),
        }?;

        let key = options.key;

        // fill in the Report Id
        let frame = {
            let mut frame = [0; 9];
            frame[1..9].copy_from_slice(&key);
            frame
        };
        device.send_feature_report(&frame)?;

        let timeout = options
            .timeout
            .map(|timeout| timeout.as_millis())
            .map_or(Ok(-1), i32::try_from)
            .map_err(|_| Error::InvalidTimeout)?;

        let air_control = Self {
            device,
            key,
            timeout,
        };
        Ok(air_control)
    }

    /// Takes a measurement from the sensor.
    pub fn read(&self) -> Result<Measurement> {
        let mut data = [0; 8];
        if self.device.read_timeout(&mut data, self.timeout)? != 8 {
            return Err(Error::InvalidMessage);
        }

        let data = decrypt(data, self.key);
        let measurement = zg_co2::decode([data[0], data[1], data[2], data[3], data[4]])?;
        Ok(measurement)
    }
}

fn decrypt(mut data: [u8; 8], key: [u8; 8]) -> [u8; 8] {
    data.swap(0, 2);
    data.swap(1, 4);
    data.swap(3, 7);
    data.swap(5, 6);

    for (r, k) in data.iter_mut().zip(key.iter()) {
        *r ^= k;
    }

    let tmp = data[7] << 5;
    data[7] = data[6] << 5 | data[7] >> 3;
    data[6] = data[5] << 5 | data[6] >> 3;
    data[5] = data[4] << 5 | data[5] >> 3;
    data[4] = data[3] << 5 | data[4] >> 3;
    data[3] = data[2] << 5 | data[3] >> 3;
    data[2] = data[1] << 5 | data[2] >> 3;
    data[1] = data[0] << 5 | data[1] >> 3;
    data[0] = tmp | data[0] >> 3;

    for (r, m) in data.iter_mut().zip(b"Htemp99e".iter()) {
        *r = r.wrapping_sub(m << 4 | m >> 4);
    }

    data
}

#[derive(Debug)]
enum DevicePathType {
    Id,
    Serial(String),
    Path(CString),
}

/// Sensor open options.
///
/// Opens the first available device with the USB Vendor ID `0x04d9`
/// and Product ID `0xa052`, a `0` encryption key and a 5 seconds timeout.
///
/// Normally there's no need to change the encryption key.
#[derive(Debug)]
pub struct OpenOptions {
    path_type: DevicePathType,
    key: [u8; 8],
    timeout: Option<Duration>,
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenOptions {
    fn new() -> Self {
        Self {
            path_type: DevicePathType::Id,
            key: [0; 8],
            timeout: Some(Duration::from_secs(5)),
        }
    }

    /// Sets the serial number of the sensor device.
    pub fn with_serial(&mut self, sn: String) -> &mut Self {
        self.path_type = DevicePathType::Serial(sn);
        self
    }

    /// Sets the path to the sensor device.
    pub fn with_path(&mut self, path: CString) -> &mut Self {
        self.path_type = DevicePathType::Path(path);
        self
    }

    /// Sets the encryption key.
    pub fn with_key(&mut self, key: [u8; 8]) -> &mut Self {
        self.key = key;
        self
    }

    /// Sets the read timeout.
    pub fn timeout(&mut self, timeout: Option<Duration>) -> &mut Self {
        self.timeout = timeout;
        self
    }

    /// Opens the sensor.
    fn open(&self) -> Result<Sensor> {
        Sensor::open(self)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_decrypt() {
        let data = [0x6c, 0xa4, 0xa2, 0xb6, 0x5d, 0x9a, 0x9c, 0x08];
        let key = [0; 8];

        let data = super::decrypt(data, key);
        assert_eq!(data, [0x50, 0x04, 0x57, 0xab, 0x0d, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_open_options_send() {
        fn assert_send<T: Send>() {}
        assert_send::<super::OpenOptions>();
    }

    #[test]
    fn test_open_options_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<super::OpenOptions>();
    }
}
