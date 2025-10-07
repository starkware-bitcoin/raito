use anyhow::Error as AnyhowError;
use std::fmt;

/// Error type for Cairo serialization that wraps anyhow::Error
/// and implements serde::ser::Error
#[derive(Debug)]
pub struct CairoSerializeError(AnyhowError);

impl From<AnyhowError> for CairoSerializeError {
    fn from(err: AnyhowError) -> Self {
        CairoSerializeError(err)
    }
}

impl fmt::Display for CairoSerializeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl serde::ser::Error for CairoSerializeError {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        CairoSerializeError(AnyhowError::msg(msg.to_string()))
    }
}

impl std::error::Error for CairoSerializeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}
