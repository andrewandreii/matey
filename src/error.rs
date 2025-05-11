use std::{
    error::Error,
    fmt::{self},
};

pub type Fallible<T> = Result<T, ConfigError>;

#[derive(Debug, Clone)]
pub enum ConfigError {
    ParseError(String),
    TokenizingError(String),
}

impl ConfigError {
    pub fn parse_error(message: String) -> Self {
        ConfigError::ParseError(message)
    }

    pub fn tokenizing_error(message: String) -> Self {
        ConfigError::TokenizingError(message)
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::ParseError(message) => write!(f, "Parsing Error: {}", message),
            ConfigError::TokenizingError(message) => write!(f, "Tokenizing Error: {}", message),
        }
    }
}

impl Error for ConfigError {}

impl<T> From<ConfigError> for Result<T, ConfigError> {
    fn from(value: ConfigError) -> Self {
        Err(value)
    }
}
