//! [Serde](https://crates.io/crates/serde)-based crate for deserializing [CCL Documents](https://chshersh.com/blog/2025-01-06-the-most-elegant-configuration-language.html).
//!
//! ```text
//! /= This is a CCL document
//! title = CCL Example
//!
//! database =
//!   enabled = true
//!   ports =
//!     = 8000
//!     = 8001
//!     = 8002
//!   limits =
//!     cpu = 1500mi
//!     memory = 10Gb
//! ```
//!
//! # Features
//! - `std` (on by default): link to the `std` crate.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub(crate) mod de;
pub(crate) mod error;
pub(crate) mod parser;
pub(crate) mod position;

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
