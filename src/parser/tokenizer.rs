use crate::error::{ConfigError, Fallible};

use super::common::FileLocation;

use std::ffi::OsString;
use std::iter::Peekable;
use std::str::CharIndices;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token<'a> {
    pub source: &'a str,
    pub location: FileLocation,
}

impl<'source> Token<'source> {
    fn new(source: &'source str, location: FileLocation) -> Self {
        Token { source, location }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ConfigToken<'a> {
    OptionCommand(Token<'a>),
    Id(Token<'a>),
    Number(Token<'a>),
    TemplateBlock(Token<'a>),
    NewLine,
    Eof,
}

struct Tokenizer<'a> {
    path: OsString,
    source: &'a str,
    iter: Peekable<CharIndices<'a>>,
    location: FileLocation,
}

impl<'source> Tokenizer<'source> {
    fn new(source: &'source str, path: OsString) -> Self {
        Tokenizer {
            path,
            source,
            iter: source.char_indices().peekable(),
            location: FileLocation::new(0, 0),
        }
    }

    fn tokenize_option_command(&mut self) -> Fallible<ConfigToken<'source>> {
        let start = self.expect('#')? + 1;

        let mut end = start - 1;
        while let Some(_) = self.iter.next_if(|(_, c)| !c.is_whitespace()) {
            self.location.step();
            end += 1;
        }

        if end <= start {
            return self.error("Expected command after #".to_string()).into();
        }

        Ok(ConfigToken::OptionCommand(Token::new(
            &self.source[start..=end],
            self.location,
        )))
    }

    fn tokenize_id(&mut self) -> Fallible<ConfigToken<'source>> {
        let start = if let Some((i, _)) = self.iter.next() {
            i
        } else {
            unreachable!();
        };

        let mut end = start;
        while let Some(_) = self.iter.next_if(|(_, c)| !c.is_whitespace()) {
            self.location.step();
            end += 1;
        }

        Ok(ConfigToken::Id(Token::new(
            &self.source[start..=end],
            self.location,
        )))
    }

    fn tokenize_template_block(&mut self) -> Fallible<ConfigToken<'source>> {
        let start = self.expect('{')?;

        let start = if let Some((i, _)) = self.iter.next_if(|(_, c)| *c == '\n') {
            i
        } else {
            start
        } + 1;

        let mut end = start - 1;
        let mut opened = 0;
        while let Some((i, c)) = self.iter.next() {
            match c {
                '{' => {
                    opened += 1;
                }
                '}' => {
                    if opened == 0 {
                        end = i;
                        break;
                    }
                    opened -= 1;
                }
                _ => {
                    continue;
                }
            }
        }

        if end <= start {
            return self.error("No matching left brace".to_string()).into();
        }

        Ok(ConfigToken::TemplateBlock(Token::new(
            &self.source[start..end],
            self.location,
        )))
    }

    fn tokenize_next(&mut self) -> Fallible<ConfigToken<'source>> {
        while let Some((_, c)) = self.iter.peek() {
            match c {
                '#' => {
                    return Ok(self.tokenize_option_command()?);
                }
                '{' => {
                    return Ok(self.tokenize_template_block()?);
                }
                '-' | '0'..'9' => {
                    todo!();
                }
                '\n' => {
                    self.location.nl();
                    self.iter.next();
                    while let Some(_) = self.iter.next_if(|(_, c)| *c == '\n') {}
                    return Ok(ConfigToken::NewLine);
                }
                c if c.is_whitespace() => {
                    self.iter.next();
                }
                _ => {
                    return Ok(self.tokenize_id()?);
                }
            }
        }

        Ok(ConfigToken::Eof)
    }

    fn expect(&mut self, c: char) -> Fallible<usize> {
        match self.iter.next() {
            Some((i, got)) if got == c => Ok(i),
            Some((_, got)) => self.error(format!("expected {}, got {}", c, got)).into(),
            None => self.error(format!("expected {}, got EOF", c)).into(),
        }
    }

    fn error(&self, reason: String) -> ConfigError {
        ConfigError::tokenizing_error(format!("({}) {}", self.location(), reason))
    }

    fn location(&self) -> String {
        format!("in {:#?} at {}", self.path, self.location)
    }
}

pub fn parse_source<'source, 'other>(
    source: &'source str,
    path: OsString,
) -> Fallible<Vec<ConfigToken<'source>>> {
    let mut tokens = Vec::with_capacity(100);

    let mut tokenizer = Tokenizer::new(source, path);

    loop {
        match tokenizer.tokenize_next()? {
            ConfigToken::Eof => break,
            token => tokens.push(token),
        }
    }

    Ok(tokens)
}
