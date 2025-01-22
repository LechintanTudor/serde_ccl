mod slice_parser;
mod str_parser;

pub(crate) use self::slice_parser::*;
pub(crate) use self::str_parser::*;

use crate::error::Result;
use core::str;

pub(crate) trait Parser<'a> {
    fn parse_key(&mut self) -> Result<&'a str>;

    fn parse_value(&mut self) -> Result<&'a str>;

    fn skip_whitespace(&mut self) -> Result<IndentState>;

    #[must_use]
    fn data(&self) -> &'a [u8];

    #[must_use]
    fn last_key_indent(&self) -> u32;

    #[must_use]
    unsafe fn position_of_ptr(&self, ptr: *const u8) -> Position {
        #[allow(clippy::cast_sign_loss)]
        let index = ptr.offset_from(self.data().as_ptr()) as usize;

        self.position_of_index(index)
    }

    #[must_use]
    fn position_of_index(&self, index: usize) -> Position {
        let data = self.data();
        let start_of_line =
            memchr::memrchr(b'\n', &data[..index]).map_or(0, |position| position + 1);

        Position {
            line: 1 + memchr::memchr_iter(b'\n', &data[..start_of_line]).count(),
            column: 1 + index - start_of_line,
        }
    }
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
