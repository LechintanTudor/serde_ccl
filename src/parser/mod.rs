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
    type Bookmark;

    fn parse_key<'s>(
        &'s mut self,
        scratch: &'s mut Vec<u8>,
    ) -> Result<(Self::Bookmark, Reference<'a, 's, str>)>;

    fn parse_value<'s>(
        &'s mut self,
        scratch: &'s mut Vec<u8>,
    ) -> Result<(Self::Bookmark, Reference<'a, 's, str>)>;

    fn skip_whitespace(&mut self, scratch: &mut Vec<u8>) -> Result<IndentState>;

    #[must_use]
    fn last_key_indent(&self) -> u32;

    #[must_use]
    fn position_of_bookmark(&self, bookmark: Self::Bookmark) -> Position;
}

#[derive(Debug)]
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

#[derive(Clone, Copy, Debug)]
pub(crate) struct Position {
    pub line: usize,
    pub column: usize,
}
