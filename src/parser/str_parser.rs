use crate::error::Result;
use crate::parser::{IndentState, Parser, Position, SliceParser};
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
    fn parse_key(&mut self) -> Result<(usize, &'a str)> {
        let (index, raw_key) = self.delegate.parse_key_raw()?;
        let key = unsafe { str::from_utf8_unchecked(raw_key) };
        Ok((index, key))
    }

    fn parse_value<'s>(&mut self) -> Result<(usize, &'a str)> {
        let (index, raw_value) = self.delegate.parse_value_raw();
        let value = unsafe { str::from_utf8_unchecked(raw_value) };
        Ok((index, value))
    }

    fn skip_whitespace(&mut self) -> Result<IndentState> {
        Ok(self.delegate.skip_whitespace_raw())
    }

    fn last_key_indent(&self) -> u32 {
        self.delegate.last_key_indent()
    }

    fn position_of_index(&self, index: usize) -> Position {
        self.delegate.position_of_index(index)
    }
}
