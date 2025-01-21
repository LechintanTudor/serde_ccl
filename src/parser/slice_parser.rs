use crate::error::{Error, ErrorKind, Result};
use crate::parser::{IndentState, Parser, Position, Reference};
use core::str;

#[must_use]
pub(crate) struct SliceParser<'a> {
    data: &'a [u8],
    index: usize,
    last_key_indent: u32,
    indent_state: IndentState,
}

impl<'a> SliceParser<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            index: 0,
            last_key_indent: 0,
            indent_state: IndentState::Start(0),
        }
    }

    pub fn parse_key_raw(&mut self) -> Result<(usize, &'a [u8])> {
        if let IndentState::Start(indent) = self.skip_whitespace_raw() {
            self.indent_state = IndentState::Middle;
            self.last_key_indent = indent;
        }

        let key_start = self.index;
        let mut found_eq = false;

        while self.index < self.data.len() {
            if self.data[self.index] == b'=' {
                found_eq = true;
                break;
            }

            self.index += 1;
        }

        if !found_eq {
            let position = self.position_of_bookmark(self.index);
            return Err(Error::new(ErrorKind::ExpectedEq, position));
        }

        let key_end = self.index;
        self.index += 1;

        Ok(trim(self.data, key_start, key_end))
    }

    pub fn parse_value_raw(&mut self) -> (usize, &'a [u8]) {
        let value_start = self.index;

        while self.index < self.data.len() {
            match self.skip_whitespace_raw() {
                IndentState::Start(indent) => {
                    if indent <= self.last_key_indent {
                        break;
                    }

                    self.indent_state = IndentState::Middle;
                }
                IndentState::Middle => (),
                IndentState::Eof => break,
            }

            while self.index < self.data.len() {
                if self.data[self.index] == b'\n' {
                    break;
                }

                self.index += 1;
            }
        }

        trim(self.data, value_start, self.index)
    }

    pub fn skip_whitespace_raw(&mut self) -> IndentState {
        while self.index < self.data.len() {
            match self.data[self.index] {
                b' ' => {
                    if let IndentState::Start(ref mut indent) = self.indent_state {
                        *indent += 1;
                    }
                }
                b'\n' => {
                    self.indent_state = IndentState::Start(0);
                }
                _ => {
                    return self.indent_state;
                }
            }

            self.index += 1;
        }

        self.indent_state = IndentState::Eof;
        self.indent_state
    }
}

impl<'a> Parser<'a> for SliceParser<'a> {
    type Bookmark = usize;

    fn parse_key<'s>(
        &'s mut self,
        _scratch: &'s mut Vec<u8>,
    ) -> Result<(Self::Bookmark, Reference<'a, 's, str>)> {
        let (bookmark, key) = self.parse_key_raw()?;

        match str::from_utf8(key) {
            Ok(key) => Ok((bookmark, Reference::Borrowed(key))),
            Err(e) => {
                let position = self.position_of_bookmark(bookmark + e.valid_up_to());
                Err(Error::new(ErrorKind::InvalidUtf8, position))
            }
        }
    }

    fn parse_value<'s>(
        &'s mut self,
        _scratch: &'s mut Vec<u8>,
    ) -> Result<(Self::Bookmark, Reference<'a, 's, str>)> {
        let (bookmark, value) = self.parse_value_raw();

        match str::from_utf8(value) {
            Ok(key) => Ok((bookmark, Reference::Borrowed(key))),
            Err(e) => {
                let position = self.position_of_bookmark(bookmark + e.valid_up_to());
                Err(Error::new(ErrorKind::InvalidUtf8, position))
            }
        }
    }

    fn skip_whitespace(&mut self, _scratch: &mut Vec<u8>) -> Result<IndentState> {
        Ok(self.skip_whitespace_raw())
    }

    fn last_key_indent(&self) -> u32 {
        self.last_key_indent
    }

    fn position_of_bookmark(&self, bookmark: Self::Bookmark) -> Position {
        // From serde_json: https://github.com/serde-rs/json

        let start_of_line =
            memchr::memrchr(b'\n', &self.data[..bookmark]).map_or(0, |position| position + 1);

        Position {
            line: 1 + memchr::memchr_iter(b'\n', &self.data[..start_of_line]).count(),
            column: 1 + bookmark - start_of_line,
        }
    }
}

#[must_use]
fn trim(data: &[u8], mut start: usize, mut end: usize) -> (usize, &[u8]) {
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

    (start, &data[start..end])
}
