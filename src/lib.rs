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
pub(crate) mod ser;

#[doc(inline)]
pub use crate::{
    de::{from_slice, from_str},
    error::{Error, ErrorKind, Result},
    ser::to_string,
};

use crate::parser::{SliceParser, StrParser};
