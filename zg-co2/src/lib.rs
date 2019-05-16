#![doc(html_root_url = "https://docs.rs/zg-co2/0.1.0-alpha.2")]
#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

//! A `no_std` crate implementing the [ZyAura ZG][ZG] CO₂ sensor protocol.
//!
//! The implementation was tested using a [TFA-Dostmann AIRCO2TROL MINI]
//! [AIRCO2TROL MINI] sensor.
//!
//! To read data from one of the compatible commercially-available USB
//! sensors, use the [`co2mon`][co2mon] crate.
//!
//! [AIRCO2TROL MINI]: https://www.tfa-dostmann.de/en/produkt/co2-monitor-airco2ntrol-mini/
//! [ZG]: http://www.zyaura.com/products/ZG_module.asp
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

/// Result type for the `decode` function.
pub type Result<T> = result::Result<T, Error>;

/// A sensor measurement.
#[derive(Debug)]
pub enum Measurement {
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
/// ```
/// let decoded = zg_co2::decode([0x50, 0x04, 0x57, 0xab, 0x0d]);
/// ```
pub fn decode(data: [u8; 5]) -> Result<Measurement> {
    if data[4] != 0x0d {
        return Err(Error::InvalidMessage);
    }

    if data[0].wrapping_add(data[1]).wrapping_add(data[2]) != data[3] {
        return Err(Error::Checksum);
    }

    let value = u16::from(data[1]) << 8 | u16::from(data[2]);
    let measurement = match data[0] {
        b'A' => Measurement::Humidity(f32::from(value) * 0.01),
        b'B' => Measurement::Temperature(f32::from(value) * 0.0625 - 273.15),
        b'P' => Measurement::CO2(value),
        _ => Measurement::Unknown(data[0], value),
    };
    Ok(measurement)
}

#[cfg(test)]
mod tests {
    use super::{Error, Measurement};
    use assert_float_eq::{afe_is_f32_near, afe_near_error_msg, assert_f32_near};

    #[test]
    fn test_decode() {
        match super::decode([0x50, 0x04, 0x57, 0xab, 0x0d]) {
            Ok(Measurement::CO2(val)) => assert_eq!(val, 1111),
            _ => assert!(false),
        }

        match super::decode([0x41, 0x00, 0x00, 0x41, 0x0d]) {
            Ok(Measurement::Humidity(val)) => assert_f32_near!(val, 0.0),
            _ => assert!(false),
        }

        match super::decode([0x42, 0x12, 0x69, 0xbd, 0x0d]) {
            Ok(Measurement::Temperature(val)) => assert_f32_near!(val, 21.4125),
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
