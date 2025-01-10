use crate::parser::Position;
use crate::Result;
use std::io;

pub(crate) trait Read {
    fn next(&mut self) -> Result<Option<u8>>;

    fn peek(&mut self) -> Result<Option<u8>>;

    #[must_use]
    fn position(&self) -> Position;

    #[must_use]
    fn peek_position(&self) -> Position;
}

#[must_use]
pub(crate) struct IoRead<R> {
    reader: R,
    peeked: Option<u8>,
}

impl<R> IoRead<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            peeked: None,
        }
    }
}

impl<R> Read for IoRead<R>
where
    R: io::Read,
{
    fn next(&mut self) -> Result<Option<u8>> {
        if let Some(byte) = self.peeked.take() {
            return Ok(Some(byte));
        }

        todo!()
    }

    fn peek(&mut self) -> Result<Option<u8>> {
        todo!()
    }

    fn position(&self) -> Position {
        todo!()
    }

    fn peek_position(&self) -> Position {
        todo!()
    }
}
