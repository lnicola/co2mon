use core::fmt::{self, Display, Formatter};

/// A possible error value when decoding a sensor message.
///
/// # Example
///
/// ```
/// let decoded = zg_co2::decode([0x50, 0x04, 0x57, 0xab, 0x00]);
/// assert!(decoded.is_err());
/// ```
#[derive(Debug)]
pub enum Error {
    /// The message was invalid (did not finish with `0x0d`).
    InvalidMessage,
    /// The message had a checksum error.
    Checksum,
    /// Hint against exhaustive matching.
    ///
    /// This enum may be extended with additional variants, so users should not
    /// count on exhaustive matching.
    #[doc(hidden)]
    __Nonexhaustive,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::InvalidMessage => write!(f, "invalid message"),
            Error::Checksum => write!(f, "checksum error"),
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    #[test]
    fn test_send() {
        fn assert_send<T: Send>() {}
        assert_send::<super::Error>();
    }

    #[test]
    fn test_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<super::Error>();
    }
}
