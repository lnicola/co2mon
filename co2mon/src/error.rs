use hidapi::HidError;
use std::error;
use std::fmt::{self, Display, Formatter};

/// An error that occurred when reading the sensor.
#[derive(Debug)]
pub enum Error {
    /// A hardware access error.
    Hid(HidError),
    /// The sensor returned an invalid message.
    InvalidMessage,
    /// A checksum error.
    Checksum,
}

impl From<HidError> for Error {
    fn from(err: HidError) -> Self {
        Error::Hid(err)
    }
}

impl From<zg_co2::Error> for Error {
    fn from(err: zg_co2::Error) -> Self {
        match err {
            zg_co2::Error::InvalidMessage => Error::InvalidMessage,
            zg_co2::Error::Checksum => Error::Checksum,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::InvalidMessage => write!(f, "invalid message"),
            Error::Checksum => write!(f, "checksum error"),
            Error::Hid(err) => err.fmt(f),
        }
    }
}

impl error::Error for Error {}
