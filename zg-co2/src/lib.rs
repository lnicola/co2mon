#![doc(html_root_url = "https://docs.rs/zg-co2/1.0.0")]
#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

//! A `no_std` crate implementing the [ZyAura ZG][ZG] CO₂ sensor protocol.
//!
//! This crate decodes the packets, but does not perform the decryption
//! commonly required for USB devices using this sensor. To read data from one
//! of the compatible commercially-available USB sensors, use the
//! [`co2mon`][co2mon] crate.
//!
//! The implementation was tested using a [TFA-Dostmann AIRCO2TROL MINI][AIRCO2TROL MINI]
//! sensor.
//!
//! [AIRCO2TROL MINI]: https://www.tfa-dostmann.de/en/produkt/co2-monitor-airco2ntrol-mini/
//! [ZG]: http://www.zyaura.com/products/ZG_module.asp
//!
//! # Example
//!
//! ```no_run
//! # use zg_co2::Result;
//! # fn main() -> Result<()> {
//! #
//! let packet = [0x50, 0x04, 0x57, 0xab, 0x0d];
//! let reading = zg_co2::decode(packet)?;
//! println!("{:?}", reading);
//! #
//! # Ok(())
//! # }
//! ```
//!
//! # Features
//!
//! The `std` feature, enabled by default, makes [`Error`][Error] implement the
//! [`Error`][std::error::Error] trait.
//!
//! # References
//!
//! See [this link][revspace] for more information about the protocol.
//!
//! [co2mon]: https://docs.rs/co2mon/
//! [revspace]: https://revspace.nl/CO2MeterHacking

use core::result;

pub use error::Error;

mod error;

/// A specialized [`Result`][std::result::Result] type for the [`decode`] function.
pub type Result<T> = result::Result<T, Error>;

/// A single sensor reading.
///
/// # Example
///
/// ```
/// # use zg_co2::{SingleReading, Result};
/// # fn main() -> Result<()> {
/// #
/// let decoded = zg_co2::decode([0x50, 0x04, 0x57, 0xab, 0x0d])?;
/// if let SingleReading::CO2(co2) = decoded {
///     println!("CO₂: {} ppm", co2);
/// }
/// #
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum SingleReading {
    /// Relative humidity
    Humidity(f32),
    /// Temperature in °C
    Temperature(f32),
    /// CO₂ concentration, measured in ppm
    CO2(u16),
    /// An unknown reading
    Unknown(u8, u16),
    /// Hint against exhaustive matching.
    ///
    /// This enum may be extended with additional variants, so users should not
    /// count on exhaustive matching.
    #[doc(hidden)]
    __Nonexhaustive,
}

/// Decodes a message from the sensor.
///
/// # Example
///
/// ```
/// let decoded = zg_co2::decode([0x50, 0x04, 0x57, 0xab, 0x0d]);
/// ```
///
/// # Errors
///
/// An error will be returned if the message could not be decoded.
pub fn decode(data: [u8; 5]) -> Result<SingleReading> {
    if data[4] != 0x0d {
        return Err(Error::InvalidMessage);
    }

    if data[0].wrapping_add(data[1]).wrapping_add(data[2]) != data[3] {
        return Err(Error::Checksum);
    }

    let value = u16::from(data[1]) << 8 | u16::from(data[2]);
    let reading = match data[0] {
        b'A' => SingleReading::Humidity(f32::from(value) * 0.01),
        b'B' => SingleReading::Temperature(f32::from(value) * 0.0625 - 273.15),
        b'P' => SingleReading::CO2(value),
        _ => SingleReading::Unknown(data[0], value),
    };
    Ok(reading)
}

#[cfg(test)]
mod tests {
    use super::{Error, SingleReading};
    use assert_float_eq::{afe_is_f32_near, afe_near_error_msg, assert_f32_near};

    #[test]
    fn test_decode() {
        match super::decode([0x50, 0x04, 0x57, 0xab, 0x0d]) {
            Ok(SingleReading::CO2(val)) => assert_eq!(val, 1111),
            _ => assert!(false),
        }

        match super::decode([0x41, 0x00, 0x00, 0x41, 0x0d]) {
            Ok(SingleReading::Humidity(val)) => assert_f32_near!(val, 0.0),
            _ => assert!(false),
        }

        match super::decode([0x42, 0x12, 0x69, 0xbd, 0x0d]) {
            Ok(SingleReading::Temperature(val)) => assert_f32_near!(val, 21.4125),
            _ => assert!(false),
        }

        match super::decode([0x42, 0x12, 0x69, 0xbd, 0x00]) {
            Err(Error::InvalidMessage) => {}
            _ => assert!(false),
        }

        match super::decode([0x42, 0x12, 0x69, 0x00, 0x0d]) {
            Err(Error::Checksum) => {}
            _ => assert!(false),
        }
    }
}
