use std::{
	error,
	fmt::{self},
	io,
};

pub type Fallible<T> = Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
	Parse(String),
	Tokenizing(String),
	IO(String),
}

impl Error {
	pub fn from_io(error: io::Error) -> Self {
		Error::IO(error.to_string())
	}

	pub fn parse_error(message: String) -> Self {
		Error::Parse(message)
	}

	pub fn tokenizing_error(message: String) -> Self {
		Error::Tokenizing(message)
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Error::Parse(message) => write!(f, "Parsing Error: {}", message),
			Error::Tokenizing(message) => write!(f, "Tokenizing Error: {}", message),
			Error::IO(message) => write!(f, "IO Error: {}", message),
		}
	}
}

impl error::Error for Error {}

impl<T> From<Error> for Result<T, Error> {
	fn from(value: Error) -> Self {
		Err(value)
	}
}
