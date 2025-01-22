mod slice_parser;
mod str_parser;

pub(crate) use self::slice_parser::*;
pub(crate) use self::str_parser::*;

use crate::error::Result;
use core::str;

pub(crate) trait Parser<'a> {
    fn parse_key(&mut self) -> Result<(usize, &'a str)>;

    fn parse_value(&mut self) -> Result<(usize, &'a str)>;

    fn skip_whitespace(&mut self) -> Result<IndentState>;

    #[must_use]
    fn last_key_indent(&self) -> u32;

    #[must_use]
    fn position_of_index(&self, index: usize) -> Position;
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum IndentState {
    Start(u32),
    Middle,
    Eof,
}

#[derive(Clone, Copy, Default, Debug)]
pub(crate) struct Position {
    pub line: usize,
    pub column: usize,
}
