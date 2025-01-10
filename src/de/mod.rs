mod deserializer;
mod read;

pub(crate) use self::deserializer::*;
pub(crate) use self::read::*;

use crate::error::Result;
use crate::parser::{ReadParser, SliceParser, StrParser};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::io;

pub fn from_reader<R, T>(reader: R) -> Result<T>
where
    R: io::Read,
    T: DeserializeOwned,
{
    let parser = ReadParser::new(IoRead::new(reader));
    T::deserialize(&mut Deserializer::new(parser))
}

pub fn from_slice<'a, T>(data: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let parser = SliceParser::new(data);
    T::deserialize(&mut Deserializer::new(parser))
}

pub fn from_str<'a, T>(data: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let parser = StrParser::new(data);
    T::deserialize(&mut Deserializer::new(parser))
}
