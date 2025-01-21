use crate::parser::Position;
use core::error::Error as CoreError;
use core::fmt;
use serde::de;

pub type Result<T> = ::core::result::Result<T, Error>;

pub struct Error(Box<ErrorImpl>);

pub(crate) struct ErrorImpl {
    kind: ErrorKind,
    line: usize,
    column: usize,
}

pub(crate) enum ErrorKind {
    Message(Box<str>),
    ExpectedEq,
    InvalidUtf8,

    // Semantic errors.
    InvalidBool,
    InvalidInt,
    InvalidFloat,
    InvalidChar,
}

impl Error {
    pub(crate) fn new(kind: ErrorKind, position: Position) -> Self {
        Self(Box::new(ErrorImpl {
            kind,
            line: position.line,
            column: position.column,
        }))
    }
}

impl de::Error for Error {
    fn custom<T>(message: T) -> Self
    where
        T: fmt::Display,
    {
        Self(Box::new(ErrorImpl {
            kind: ErrorKind::Message(message.to_string().into_boxed_str()),
            line: 0,
            column: 0,
        }))
    }
}

impl CoreError for Error {
    // Empty
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Error({:?}, line: {}, column: {})",
            self.0.kind.to_string(),
            self.0.line,
            self.0.column
        )
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at line {} column {}",
            self.0.kind, self.0.line, self.0.column
        )
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::Message(message) => f.write_str(&message),
            ErrorKind::ExpectedEq => f.write_str("expected equal sign"),
            ErrorKind::InvalidUtf8 => f.write_str("invalid UTF-8"),
            ErrorKind::InvalidBool => f.write_str("invalid bool"),
            ErrorKind::InvalidInt => f.write_str("invalid int"),
            ErrorKind::InvalidFloat => f.write_str("invalid float"),
            ErrorKind::InvalidChar => f.write_str("invalid char"),
        }
    }
}
