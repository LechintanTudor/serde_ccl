use crate::position::Position;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use core::error::Error as CoreError;
use core::fmt;
use serde::de;

/// Result type returned by functions that can fail.
pub type Result<T> = ::core::result::Result<T, Error>;

/// Error type returned by functions that can fail.
pub struct Error(Box<ErrorImpl>);

pub(crate) struct ErrorImpl {
    code: ErrorCode,
    position: Position,
}

pub(crate) enum ErrorCode {
    // Parser errors.
    ExpectedEq,
    InvalidUtf8,

    // Semantic errors.
    Message(String),
    InvalidBool,
    InvalidInt,
    InvalidFloat,
    InvalidChar,
}

/// The kind of error.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ErrorKind {
    /// The input is not a valid CCL document.
    Syntax,

    /// The deserialized data is semantically incorrect.
    Semantic,
}

impl Error {
    #[must_use]
    pub(crate) fn new(code: ErrorCode, position: Position) -> Self {
        Self(Box::new(ErrorImpl { code, position }))
    }

    #[inline]
    #[must_use]
    pub(crate) fn with_position(mut self, position: Position) -> Self {
        self.0.position = position;
        self
    }

    /// Returns the kind of error that occurred.
    #[inline]
    #[must_use]
    pub fn kind(&self) -> ErrorKind {
        self.0.code.kind()
    }

    #[inline]
    #[must_use]
    pub(crate) fn position(&self) -> Position {
        self.0.position
    }

    /// Returns the line at which the error occurred.
    #[inline]
    #[must_use]
    pub fn line(&self) -> usize {
        self.0.position.line
    }

    /// Returns the column at which the error occurred.
    #[inline]
    #[must_use]
    pub fn column(&self) -> usize {
        self.0.position.column
    }
}

impl de::Error for Error {
    fn custom<T>(message: T) -> Self
    where
        T: fmt::Display,
    {
        Self(Box::new(ErrorImpl {
            code: ErrorCode::Message(message.to_string()),
            position: Position::default(),
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
            self.0.code.to_string(),
            self.0.position.line,
            self.0.position.column,
        )
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at line {} column {}",
            self.0.code, self.0.position.line, self.0.position.column,
        )
    }
}

impl ErrorCode {
    #[must_use]
    fn kind(&self) -> ErrorKind {
        #[allow(clippy::match_same_arms)]
        match self {
            Self::Message(_) => ErrorKind::Syntax,
            Self::ExpectedEq => ErrorKind::Syntax,
            Self::InvalidUtf8 => ErrorKind::Semantic,
            Self::InvalidBool => ErrorKind::Semantic,
            Self::InvalidInt => ErrorKind::Semantic,
            Self::InvalidFloat => ErrorKind::Semantic,
            Self::InvalidChar => ErrorKind::Semantic,
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCode::Message(message) => f.write_str(message),
            ErrorCode::ExpectedEq => f.write_str("expected equal sign"),
            ErrorCode::InvalidUtf8 => f.write_str("invalid UTF-8"),
            ErrorCode::InvalidBool => f.write_str("invalid bool"),
            ErrorCode::InvalidInt => f.write_str("invalid int"),
            ErrorCode::InvalidFloat => f.write_str("invalid float"),
            ErrorCode::InvalidChar => f.write_str("invalid char"),
        }
    }
}
