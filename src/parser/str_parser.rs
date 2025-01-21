use crate::error::Result;
use crate::parser::{IndentState, Parser, Position, Reference, SliceParser};
use core::str;

#[must_use]
pub(crate) struct StrParser<'a> {
    delegate: SliceParser<'a>,
}

impl<'a> StrParser<'a> {
    pub fn new(data: &'a str) -> Self {
        Self {
            delegate: SliceParser::new(data.as_bytes()),
        }
    }
}

impl<'a> Parser<'a> for StrParser<'a> {
    type Bookmark = usize;

    fn parse_key<'s>(
        &'s mut self,
        _scratch: &'s mut Vec<u8>,
    ) -> Result<(Self::Bookmark, Reference<'a, 's, str>)> {
        let (bookmark, raw_key) = self.delegate.parse_key_raw()?;
        let key = unsafe { str::from_utf8_unchecked(raw_key) };
        Ok((bookmark, Reference::Borrowed(key)))
    }

    fn parse_value<'s>(
        &'s mut self,
        _scratch: &'s mut Vec<u8>,
    ) -> Result<(Self::Bookmark, Reference<'a, 's, str>)> {
        let (bookmark, raw_value) = self.delegate.parse_value_raw();
        let value = unsafe { str::from_utf8_unchecked(raw_value) };
        Ok((bookmark, Reference::Borrowed(value)))
    }

    fn skip_whitespace(&mut self, _scratch: &mut Vec<u8>) -> Result<IndentState> {
        Ok(self.delegate.skip_whitespace_raw())
    }

    fn last_key_indent(&self) -> u32 {
        self.delegate.last_key_indent()
    }

    fn position_of_bookmark(&self, bookmark: Self::Bookmark) -> Position {
        self.delegate.position_of_bookmark(bookmark)
    }
}
