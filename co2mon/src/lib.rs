#![doc(html_root_url = "https://docs.rs/co2mon/2.0.3")]
#![deny(missing_docs)]

//! A driver for the Holtek ([ZyAura ZG][ZG]) CO₂ USB monitors.
//!
//! The implementation was tested using a
//! [TFA-Dostmann AIRCO2NTROL MINI][AIRCO2NTROL MINI] sensor.
//!
//! [AIRCO2NTROL MINI]: https://www.tfa-dostmann.de/en/produkt/co2-monitor-airco2ntrol-mini/
//! [ZG]: http://www.zyaura.com/products/ZG_module.asp
//!
//! # Example usage
//!
//! ```no_run
//! # use co2mon::{Result, Sensor};
//! # fn main() -> Result<()> {
//! #
//! let sensor = Sensor::open_default()?;
//! let reading = sensor.read_one()?;
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
//!
//! The USB HID protocol is not documented, but was [reverse-engineered][had] [before][revspace].
//!
//! [had]: https://hackaday.io/project/5301/
//! [revspace]: https://revspace.nl/CO2MeterHacking

use hidapi::{HidApi, HidDevice};
use std::convert::TryFrom;
use std::ffi::CString;
use std::result;
use std::time::{Duration, Instant};

pub use error::Error;
pub use zg_co2::SingleReading;

mod error;

/// A specialized [`Result`][std::result::Result] type for the fallible functions.
pub type Result<T> = result::Result<T, Error>;

/// A reading consisting of temperature (in °C) and CO₂ concentration (in ppm) values.
///
/// # Example
///
/// ```no_run
/// # use co2mon::{Result, Sensor};
/// # fn main() -> Result<()> {
/// #
/// let sensor = Sensor::open_default()?;
/// let reading = sensor.read()?;
/// println!("{} °C, {} ppm CO₂", reading.temperature(), reading.co2());
/// #
/// # Ok(())
/// # }
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Reading {
    temperature: f32,
    co2: u16,
}

impl Reading {
    /// Returns the measured temperature in °C.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use co2mon::{Result, Sensor};
    /// # fn main() -> Result<()> {
    /// #
    /// let sensor = Sensor::open_default()?;
    /// let reading = sensor.read()?;
    /// println!("{} °C", reading.temperature());
    /// #
    /// # Ok(())
    /// # }
    pub fn temperature(&self) -> f32 {
        self.temperature
    }

    /// Returns the CO₂ concentration in ppm (parts per million).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use co2mon::{Result, Sensor};
    /// # fn main() -> Result<()> {
    /// #
    /// let sensor = Sensor::open_default()?;
    /// let reading = sensor.read()?;
    /// println!("{} ppm CO₂", reading.co2());
    /// #
    /// # Ok(())
    /// # }
    pub fn co2(&self) -> u16 {
        self.co2
    }
}

/// Sensor driver struct.
///
/// # Example
///
/// ```no_run
/// # use co2mon::{Result, Sensor};
/// # fn main() -> Result<()> {
/// #
/// let sensor = Sensor::open_default()?;
/// let reading = sensor.read_one()?;
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
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use co2mon::{Result, Sensor};
    /// # fn main() -> Result<()> {
    /// #
    /// let sensor = Sensor::open_default()?;
    /// let reading = sensor.read_one()?;
    /// println!("{:?}", reading);
    /// #
    /// # Ok(())
    /// # }
    pub fn open_default() -> Result<Self> {
        OpenOptions::new().open()
    }

    fn open(options: &OpenOptions) -> Result<Self> {
        let hidapi = HidApi::new()?;

        const VID: u16 = 0x04d9;
        const PID: u16 = 0xa052;

        let device = match options.path_type {
            DevicePathType::Id => hidapi.open(VID, PID),
            DevicePathType::SerialNumber(ref sn) => hidapi.open_serial(VID, PID, sn),
            DevicePathType::Path(ref path) => hidapi.open_path(path),
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

    /// Takes a single reading from the sensor.
    ///
    /// # Errors
    ///
    /// An error will be returned on an I/O error or if a message could not be
    /// read or decoded.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use co2mon::{Result, Sensor};
    /// # fn main() -> Result<()> {
    /// #
    /// let sensor = Sensor::open_default()?;
    /// let reading = sensor.read_one()?;
    /// println!("{:?}", reading);
    /// #
    /// # Ok(())
    /// # }
    pub fn read_one(&self) -> Result<SingleReading> {
        let mut data = [0; 8];
        if self.device.read_timeout(&mut data, self.timeout)? != 8 {
            return Err(Error::InvalidMessage);
        }

        // if the "magic byte" is present no decryption is necessary. This is the case for AIRCO2NTROL COACH
        // and newer AIRCO2NTROL MINIs in general
        let data = if data[4] == 0x0d {
            data
        } else {
            decrypt(data, self.key)
        };
        let reading = zg_co2::decode([data[0], data[1], data[2], data[3], data[4]])?;
        Ok(reading)
    }

    /// Takes a multiple readings from the sensor until the temperature and
    /// CO₂ concentration are available, and returns both.
    ///
    /// # Errors
    ///
    /// An error will be returned on an I/O error or if a message could not be
    /// read or decoded.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use co2mon::{Result, Sensor};
    /// # fn main() -> Result<()> {
    /// #
    /// let sensor = Sensor::open_default()?;
    /// let reading = sensor.read()?;
    /// println!("{} °C, {} ppm CO₂", reading.temperature(), reading.co2());
    /// #
    /// # Ok(())
    /// # }
    pub fn read(&self) -> Result<Reading> {
        let start = Instant::now();
        let mut temperature = None;
        let mut co2 = None;
        loop {
            let reading = self.read_one()?;
            match reading {
                SingleReading::Temperature(val) => temperature = Some(val),
                SingleReading::CO2(val) => co2 = Some(val),
                _ => {}
            }
            if let (Some(temperature), Some(co2)) = (temperature, co2) {
                let reading = Reading { temperature, co2 };
                return Ok(reading);
            }

            if self.timeout != -1 {
                let duration = Instant::now() - start;
                if duration.as_millis() > self.timeout as u128 {
                    return Err(Error::Timeout);
                }
            }
        }
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

#[derive(Debug, Clone)]
enum DevicePathType {
    Id,
    SerialNumber(String),
    Path(CString),
}

/// Sensor open options.
///
/// Opens the first available device with the USB Vendor ID `0x04d9`
/// and Product ID `0xa052`, a `0` encryption key and a 5 seconds timeout.
///
/// Normally there's no need to change the encryption key.
///
/// # Example
///
/// ```no_run
/// # use co2mon::{OpenOptions, Result};
/// # use std::time::Duration;
/// # fn main() -> Result<()> {
/// #
/// let sensor = OpenOptions::new()
///     .timeout(Some(Duration::from_secs(10)))
///     .open()?;
/// #
/// # Ok(())
/// # }
#[derive(Debug, Clone)]
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
    /// Creates a new set of options to be configured.
    ///
    /// The defaults are opening the first connected sensor and a timeout of
    /// 5 seconds.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use co2mon::{OpenOptions, Result};
    /// # use std::time::Duration;
    /// # fn main() -> Result<()> {
    /// #
    /// let sensor = OpenOptions::new()
    ///     .timeout(Some(Duration::from_secs(10)))
    ///     .open()?;
    /// #
    /// # Ok(())
    /// # }
    pub fn new() -> Self {
        Self {
            path_type: DevicePathType::Id,
            key: [0; 8],
            timeout: Some(Duration::from_secs(5)),
        }
    }

    /// Sets the serial number of the sensor device to open.
    ///
    /// The serial number appears to be the firmware version.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use co2mon::OpenOptions;
    /// # use std::error::Error;
    /// # use std::ffi::CString;
    /// # use std::result::Result;
    /// # fn main() -> Result<(), Box<Error>> {
    /// #
    /// let sensor = OpenOptions::new()
    ///     .with_serial_number("1.40")
    ///     .open()?;
    /// #
    /// # Ok(())
    /// # }
    pub fn with_serial_number<S: Into<String>>(&mut self, sn: S) -> &mut Self {
        self.path_type = DevicePathType::SerialNumber(sn.into());
        self
    }

    /// Sets the path to the sensor device to open.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use co2mon::OpenOptions;
    /// # use std::error::Error;
    /// # use std::ffi::CString;
    /// # use std::result::Result;
    /// # fn main() -> Result<(), Box<Error>> {
    /// #
    /// let sensor = OpenOptions::new()
    ///     .with_path(CString::new("/dev/bus/usb/001/004")?)
    ///     .open()?;
    /// #
    /// # Ok(())
    /// # }
    pub fn with_path(&mut self, path: CString) -> &mut Self {
        self.path_type = DevicePathType::Path(path);
        self
    }

    /// Sets the encryption key.
    ///
    /// The key is used to encrypt the communication with the sensor, but
    /// changing it is probably not very useful.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use co2mon::{OpenOptions, Result};
    /// # use std::ffi::CString;
    /// # fn main() -> Result<()> {
    /// #
    /// let sensor = OpenOptions::new()
    ///     .with_key([0x62, 0xea, 0x1d, 0x4f, 0x14, 0xfa, 0xe5, 0x6c])
    ///     .open()?;
    /// #
    /// # Ok(())
    /// # }
    pub fn with_key(&mut self, key: [u8; 8]) -> &mut Self {
        self.key = key;
        self
    }

    /// Sets the read timeout.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use co2mon::{OpenOptions, Result};
    /// # use std::time::Duration;
    /// # fn main() -> Result<()> {
    /// #
    /// let sensor = OpenOptions::new()
    ///     .timeout(Some(Duration::from_secs(10)))
    ///     .open()?;
    /// #
    /// # Ok(())
    /// # }
    pub fn timeout(&mut self, timeout: Option<Duration>) -> &mut Self {
        self.timeout = timeout;
        self
    }

    /// Opens the sensor.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use co2mon::{OpenOptions, Result};
    /// # use std::time::Duration;
    /// # fn main() -> Result<()> {
    /// #
    /// let sensor = OpenOptions::new()
    ///     .timeout(Some(Duration::from_secs(10)))
    ///     .open()?;
    /// #
    /// # Ok(())
    /// # }
    pub fn open(&self) -> Result<Sensor> {
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
        assert_send::<super::Sensor>();
        assert_send::<super::OpenOptions>();
    }

    #[test]
    fn test_open_options_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<super::OpenOptions>();
    }
}
