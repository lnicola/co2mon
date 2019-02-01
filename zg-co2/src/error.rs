use core::fmt::{self, Display, Formatter};

/// An error that occurred during decoding the message.
#[derive(Debug)]
pub enum Error {
    /// The message was invalid (did not finish with `0x0d`).
    InvalidMessage,
    /// The message had a checksum error.
    Checksum,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::InvalidMessage => write!(f, "invalid message"),
            Error::Checksum => write!(f, "checksum error"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
