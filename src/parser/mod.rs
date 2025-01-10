mod read_parser;
mod slice_parser;
mod str_parser;

pub(crate) use self::read_parser::*;
pub(crate) use self::slice_parser::*;
pub(crate) use self::str_parser::*;

use crate::error::Result;
use core::ops::Deref;
use core::str;

pub(crate) trait Parser<'a> {
    fn parse_key<'s>(&'s mut self, scratch: &'s mut Vec<u8>) -> Result<Reference<'a, 's, str>>;

    fn parse_value<'s>(&'s mut self, scratch: &'s mut Vec<u8>) -> Result<Reference<'a, 's, str>>;

    fn skip_whitespace(&mut self, scratch: &mut Vec<u8>) -> Result<IndentState>;

    #[must_use]
    fn last_key_indent(&self) -> Option<u32>;
}

pub(crate) enum Reference<'b, 'c, T>
where
    T: ?Sized,
{
    Borrowed(&'b T),
    Copied(&'c T),
}

impl<T> Deref for Reference<'_, '_, T>
where
    T: ?Sized,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(b) => b,
            Self::Copied(c) => c,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum IndentState {
    Start(u32),
    Middle,
    Eof,
}

#[must_use]
pub(crate) fn trim(data: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = data.len();

    while start < end {
        let byte = data[start];

        if !(byte == b' ' || byte == b'\n') {
            break;
        }

        start += 1;
    }

    while end > start {
        let byte = data[end - 1];

        if !(byte == b' ' || byte == b'\n') {
            break;
        }

        end -= 1;
    }

    &data[start..end]
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Position {
    pub line: usize,
    pub column: usize,
}
