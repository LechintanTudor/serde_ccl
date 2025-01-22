pub(crate) mod de;
pub(crate) mod error;
pub(crate) mod parser;

#[doc(inline)]
pub use crate::error::{Error, ErrorKind, Result};

use crate::de::Deserializer;
use crate::parser::{SliceParser, StrParser};
use serde::de::Deserialize;

/// Deserialize the value from a byte slice.
pub fn from_slice<'a, T>(data: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let parser = SliceParser::new(data);
    T::deserialize(&mut Deserializer::new(parser))
}

/// Deserialize the value from a string.
pub fn from_str<'a, T>(data: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let parser = StrParser::new(data);
    T::deserialize(&mut Deserializer::new(parser))
}
