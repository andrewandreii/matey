use std::{
    error,
    fmt::{self},
};

pub type Fallible<T> = Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    ParseError(String),
    TokenizingError(String),
    IOError(String),
}

impl Error {
    pub fn parse_error(message: String) -> Self {
        Error::ParseError(message)
    }

    pub fn tokenizing_error(message: String) -> Self {
        Error::TokenizingError(message)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ParseError(message) => write!(f, "Parsing Error: {}", message),
            Error::TokenizingError(message) => write!(f, "Tokenizing Error: {}", message),
            Error::IOError(message) => write!(f, "IO Error: {}", message),
        }
    }
}

impl error::Error for Error {}

impl<T> From<Error> for Result<T, Error> {
    fn from(value: Error) -> Self {
        Err(value)
    }
}
