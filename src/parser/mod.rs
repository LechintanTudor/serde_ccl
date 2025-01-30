mod slice_parser;
mod str_parser;

pub(crate) use self::slice_parser::*;
pub(crate) use self::str_parser::*;

use crate::error::Result;
use crate::position::Position;
use core::str;

pub(crate) trait Parser<'a> {
    fn parse_key(&mut self) -> Result<&'a str>;

    fn parse_value(&mut self) -> Result<&'a str>;

    fn skip_whitespace(&mut self) -> Result<IndentState>;

    #[must_use]
    fn data(&self) -> &'a [u8];

    #[must_use]
    fn last_key_index(&self) -> usize;

    #[must_use]
    fn last_key_indent(&self) -> u32;

    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    unsafe fn index_of_ptr(&self, ptr: *const u8) -> usize {
        ptr.offset_from(self.data().as_ptr()) as usize
    }

    #[must_use]
    fn position_of_index(&self, index: usize) -> Position {
        // Adapted from serde_json: https://github.com/serde-rs/json.

        let data = self.data();
        let start_of_line =
            memchr::memrchr(b'\n', &data[..index]).map_or(0, |position| position + 1);

        Position {
            line: 1 + memchr::memchr_iter(b'\n', &data[..start_of_line]).count(),
            column: 1 + index - start_of_line,
        }
    }

    #[must_use]
    unsafe fn position_of_ptr(&self, ptr: *const u8) -> Position {
        self.position_of_index(self.index_of_ptr(ptr))
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum IndentState {
    Start(u32),
    Middle,
    Eof,
}
