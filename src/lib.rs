pub mod error;

pub(crate) mod de;
pub(crate) mod parser;

#[doc(inline)]
pub use crate::error::{Error, Result};

#[doc(inline)]
pub use crate::de::{from_reader, from_slice, from_str};
