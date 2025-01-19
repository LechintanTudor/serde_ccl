use crate::error::{Error, Result};
use crate::parser::{self, IndentState, Parser, Position, Reference};
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
            return Err(Error::UnexpectedChar);
        }

        let key_end = self.index;
        self.index += 1;

        Ok(parser::trim(&self.data[key_start..key_end]))
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

        parser::trim(&self.data[value_start..self.index])
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

    #[must_use]
    pub fn position_of_index(&self, index: usize) -> Position {
        let line_start = self.data[..index]
            .iter()
            .rposition(|&b| b == b'\n')
            .map_or(0, |l| l + 1);

        let line_count = self.data[..line_start]
            .iter()
            .filter(|&&c| c == b'\n')
            .count();

        Position {
            line: 1 + line_count,
            column: index - line_start,
        }
    }
}

impl<'a> Parser<'a> for SliceParser<'a> {
    fn parse_key<'s>(&'s mut self, _scratch: &'s mut Vec<u8>) -> Result<Reference<'a, 's, str>> {
        let key = self.parse_key_raw()?;
        Ok(Reference::Borrowed(str::from_utf8(key)?))
    }

    fn parse_value<'s>(&'s mut self, _scratch: &'s mut Vec<u8>) -> Result<Reference<'a, 's, str>> {
        let value = self.parse_value_raw();
        Ok(Reference::Borrowed(str::from_utf8(value)?))
    }

    fn skip_whitespace(&mut self, _scratch: &mut Vec<u8>) -> Result<IndentState> {
        Ok(self.skip_whitespace_raw())
    }

    fn last_key_indent(&self) -> u32 {
        self.last_key_indent
    }
}
