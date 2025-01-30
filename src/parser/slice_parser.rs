use crate::error::{Error, ErrorCode, Result};
use crate::parser::{IndentState, Parser};
use core::str;

#[must_use]
pub(crate) struct SliceParser<'a> {
    data: &'a [u8],
    index: usize,
    last_key_index: usize,
    last_key_indent: u32,
    indent_state: IndentState,
}

impl<'a> SliceParser<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            index: 0,
            last_key_indent: 0,
            last_key_index: 0,
            indent_state: IndentState::Start(0),
        }
    }

    pub fn parse_key_raw(&mut self) -> Result<&'a [u8]> {
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
            let position = self.position_of_index(self.index);
            return Err(Error::new(ErrorCode::ExpectedEq, position));
        }

        let key_end = self.index;
        self.index += 1;

        let key = trim(&self.data[key_start..key_end]);
        self.last_key_index = unsafe { self.index_of_ptr(key.as_ptr()) };
        Ok(key)
    }

    pub fn parse_value_raw(&mut self) -> &'a [u8] {
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

        trim(&self.data[value_start..self.index])
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
    fn parse_key(&mut self) -> Result<&'a str> {
        let key = self.parse_key_raw()?;

        str::from_utf8(key).map_err(|e| unsafe {
            Error::new(
                ErrorCode::InvalidUtf8,
                self.position_of_ptr(key.as_ptr().add(e.valid_up_to())),
            )
        })
    }

    fn parse_value(&mut self) -> Result<&'a str> {
        let value = self.parse_value_raw();

        str::from_utf8(value).map_err(|e| {
            Error::new(ErrorCode::InvalidUtf8, unsafe {
                self.position_of_ptr(value.as_ptr().add(e.valid_up_to()))
            })
        })
    }

    fn skip_whitespace(&mut self) -> Result<IndentState> {
        Ok(self.skip_whitespace_raw())
    }

    fn data(&self) -> &'a [u8] {
        self.data
    }

    fn last_key_index(&self) -> usize {
        self.last_key_index
    }

    fn last_key_indent(&self) -> u32 {
        self.last_key_indent
    }
}

#[must_use]
fn trim(data: &[u8]) -> &[u8] {
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
