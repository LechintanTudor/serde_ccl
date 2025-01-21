use crate::de::Read;
use crate::error::Result;
use crate::parser::{IndentState, Parser, Position, Reference};
use core::str;

#[must_use]
pub(crate) struct ReadParser<R> {
    reader: R,
    last_key_indent: u32,
    indent_state: IndentState,
}

impl<R> ReadParser<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            last_key_indent: 0,
            indent_state: IndentState::Start(0),
        }
    }
}

impl<'a, R> Parser<'a> for ReadParser<R>
where
    R: Read,
{
    type Bookmark = Position;

    fn parse_key<'s>(
        &'s mut self,
        scratch: &'s mut Vec<u8>,
    ) -> Result<(Self::Bookmark, Reference<'a, 's, str>)> {
        if let IndentState::Start(indent) = self.skip_whitespace(scratch)? {
            self.indent_state = IndentState::Middle;
            self.last_key_indent = indent;
        }

        scratch.clear();

        loop {
            let Some(peeked) = self.reader.peek()? else {
                todo!("Unexpected EOF");
            };

            if peeked == b'=' {
                break;
            }

            scratch.push(peeked);
            self.reader.next()?;
        }

        let eq = self.reader.next()?;
        assert_eq!(eq, Some(b'='));

        todo!()
    }

    fn parse_value<'s>(
        &'s mut self,
        scratch: &'s mut Vec<u8>,
    ) -> Result<(Self::Bookmark, Reference<'a, 's, str>)> {
        scratch.clear();

        'main: loop {
            match self.skip_whitespace(scratch)? {
                IndentState::Start(indent) => {
                    if indent <= self.last_key_indent {
                        break;
                    }

                    scratch.extend((0..indent).map(|_| b' '));
                    self.indent_state = IndentState::Middle;
                }
                IndentState::Middle => (),
                IndentState::Eof => break,
            }

            loop {
                let Some(peeked) = self.reader.peek()? else {
                    break 'main;
                };

                if peeked == b'\n' {
                    break;
                }

                scratch.push(peeked);
                self.reader.next()?;
            }
        }

        todo!()
    }

    fn skip_whitespace(&mut self, scratch: &mut Vec<u8>) -> Result<IndentState> {
        loop {
            let Some(peeked) = self.reader.peek()? else {
                self.indent_state = IndentState::Eof;
                break;
            };

            match peeked {
                b' ' => {
                    if let IndentState::Start(ref mut indent) = self.indent_state {
                        *indent += 1;
                    }
                }
                b'\n' => {
                    self.indent_state = IndentState::Start(0);
                    scratch.push(b'\n');
                }
                _ => {
                    break;
                }
            }

            self.reader.next()?;
        }

        Ok(self.indent_state)
    }

    fn last_key_indent(&self) -> u32 {
        self.last_key_indent
    }

    fn position_of_bookmark(&self, bookmark: Self::Bookmark) -> Position {
        bookmark
    }
}
