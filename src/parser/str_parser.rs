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
    fn parse_key(&mut self) -> Result<&'a str> {
        self.delegate
            .parse_key_raw()
            .map(|key| unsafe { str::from_utf8_unchecked(key) })
    }

    fn parse_value<'s>(&mut self) -> Result<&'a str> {
        let value = self.delegate.parse_value_raw();
        unsafe { Ok(str::from_utf8_unchecked(value)) }
    }

    fn skip_whitespace(&mut self) -> Result<IndentState> {
        Ok(self.delegate.skip_whitespace_raw())
    }

    fn data(&self) -> &'a [u8] {
        self.delegate.data()
    }

    fn last_key_indent(&self) -> u32 {
        self.delegate.last_key_indent()
    }

    fn position_of_index(&self, index: usize) -> Position {
        self.delegate.position_of_index(index)
    }
}
