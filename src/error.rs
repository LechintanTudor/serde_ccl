use core::char::ParseCharError;
use core::error::Error as StdError;
use core::fmt;
use core::num::{ParseFloatError, ParseIntError};
use core::str::{ParseBoolError, Utf8Error};
use serde::de;

pub type Result<T> = ::core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Utf8(Utf8Error),
    ParseBool(ParseBoolError),
    ParseInt(ParseIntError),
    ParseFloat(ParseFloatError),
    ParseChar(ParseCharError),
    UnexpectedChar,
    Serde(String),
}

impl de::Error for Error {
    fn custom<T>(message: T) -> Self
    where
        T: fmt::Display,
    {
        Self::Serde(message.to_string())
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        let source: &dyn StdError = match self {
            Self::ParseInt(e) => e,
            Self::ParseFloat(e) => e,
            _ => return None,
        };

        Some(source)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnexpectedChar => write!(f, "Unexpected character"),
            Error::Utf8(e) => write!(f, "{e}"),
            Error::ParseBool(e) => write!(f, "{e}"),
            Error::ParseInt(e) => write!(f, "{e}"),
            Error::ParseFloat(e) => write!(f, "{e}"),
            Error::ParseChar(e) => write!(f, "{e}"),
            Error::Serde(e) => write!(f, "{e}"),
        }
    }
}

impl From<Utf8Error> for Error {
    fn from(error: Utf8Error) -> Self {
        Self::Utf8(error)
    }
}

impl From<ParseBoolError> for Error {
    fn from(error: ParseBoolError) -> Self {
        Self::ParseBool(error)
    }
}

impl From<ParseIntError> for Error {
    fn from(error: ParseIntError) -> Self {
        Self::ParseInt(error)
    }
}

impl From<ParseFloatError> for Error {
    fn from(error: ParseFloatError) -> Self {
        Self::ParseFloat(error)
    }
}

impl From<ParseCharError> for Error {
    fn from(error: ParseCharError) -> Self {
        Self::ParseChar(error)
    }
}
