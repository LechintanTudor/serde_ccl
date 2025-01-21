use crate::parser::Position;
use crate::Result;
use std::io::{self, Bytes};

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
    bytes: Bytes<R>,
    peeked: Option<Option<u8>>,
}

impl<R> IoRead<R> {
    pub fn new(reader: R) -> Self
    where
        R: io::Read,
    {
        Self {
            bytes: reader.bytes(),
            peeked: None,
        }
    }
}

impl<R> Read for IoRead<R>
where
    R: io::Read,
{
    fn next(&mut self) -> Result<Option<u8>> {
        if let Some(peeked) = self.peeked.take() {
            return Ok(peeked);
        }

        match self.bytes.next() {
            Some(Ok(b)) => Ok(Some(b)),
            Some(Err(e)) => todo!(),
            None => Ok(None),
        }
    }

    fn peek(&mut self) -> Result<Option<u8>> {
        if let Some(peeked) = self.peeked {
            return Ok(peeked);
        }

        let peeked = match self.bytes.next() {
            Some(Ok(b)) => Some(b),
            Some(Err(e)) => todo!(),
            None => None,
        };

        self.peeked = Some(peeked);
        Ok(peeked)
    }

    fn position(&self) -> Position {
        todo!()
    }

    fn peek_position(&self) -> Position {
        todo!()
    }
}
