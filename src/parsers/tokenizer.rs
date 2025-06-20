use crate::error::{Error, Fallible};

use super::common::FileLocation;

use std::ffi::OsString;
use std::iter::Peekable;
use std::str::CharIndices;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Token<'a> {
	pub source: &'a str,
	pub location: FileLocation,
}

impl<'source> Token<'source> {
	fn new(source: &'source str, location: FileLocation) -> Self {
		Token { source, location }
	}
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[allow(dead_code)]
pub enum ConfigToken<'a> {
	OptionCommand(Token<'a>),
	Id(Token<'a>),
	Number(Token<'a>),
	Literal(Token<'a>),
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
			location: FileLocation::new(),
		}
	}

	fn tokenize_option_command(&mut self) -> Fallible<ConfigToken<'source>> {
		let start = self.expect('#')? + 1;

		let mut end = start - 1;
		while self.iter.next_if(|(_, c)| c.is_alphabetic()).is_some() {
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
		while self.iter.next_if(|(_, c)| c.is_alphanumeric()).is_some() {
			self.location.step();
			end += 1;
		}

		Ok(ConfigToken::Id(Token::new(
			&self.source[start..=end],
			self.location,
		)))
	}

	fn tokenize_literal(&mut self) -> Fallible<ConfigToken<'source>> {
		let (start, is_double) = if let Ok(pos) = self.expect('"') {
			(pos + 1, true)
		} else {
			(self.expect('\'')? + 1, false)
		};

		let mut is_template = false;
		let mut end = start - 1;
		for (i, c) in self.iter.by_ref() {
			self.location.step();
			match c {
				'"' => {
					if !is_double {
						continue;
					}
					end = i;
					break;
				}
				'\'' => {
					if is_double {
						continue;
					}
					end = i;
					break;
				}
				'\n' => {
					return self
						.error("No matching quotes found for string".to_string())
						.into();
				}
				'{' => {
					is_template = true;
				}
				_ => continue,
			}
		}

		if end < start {
			return self.error("String doesn't end".to_string()).into();
		}

		if is_template {
			Ok(ConfigToken::TemplateBlock(Token::new(
				&self.source[start..end],
				self.location,
			)))
		} else {
			Ok(ConfigToken::Literal(Token::new(
				&self.source[start..end],
				self.location,
			)))
		}
	}

	fn tokenize_template_block(&mut self) -> Fallible<ConfigToken<'source>> {
		let start = self.expect('{')?;

		let start = if let Some((i, _)) = self.iter.next_if(|(_, c)| *c == '\n') {
			self.location.nl();
			i
		} else {
			start
		} + 1;

		let mut end = start - 1;
		let mut opened = 0;
		for (i, c) in self.iter.by_ref() {
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
				'\n' => {
					self.location.nl();
				}
				_ => {
					self.location.step();
				}
			}
		}

		if end < start {
			return self.error("No matching left brace".to_string()).into();
		}

		Ok(ConfigToken::TemplateBlock(Token::new(
			&self.source[start..end],
			self.location,
		)))
	}

	fn tokenize_next(&mut self) -> Fallible<ConfigToken<'source>> {
		let mut next = Ok(ConfigToken::Eof);
		while let Some((_, c)) = self.iter.peek() {
			next = match c {
				'#' => self.tokenize_option_command(),
				'{' => self.tokenize_template_block(),
				'"' | '\'' => self.tokenize_literal(),
				'\n' => {
					self.location.nl();
					self.iter.next();
					while self.iter.next_if(|(_, c)| *c == '\n').is_some() {
						self.location.nl();
					}
					Ok(ConfigToken::NewLine)
				}
				c if c.is_whitespace() => {
					self.iter.next();
					continue;
				}
				c if c.is_alphabetic() => self.tokenize_id(),
				&c => self.error(format!("Unexpected token {}", c)).into(),
			};
			break;
		}

		next
	}

	fn expect(&mut self, c: char) -> Fallible<usize> {
		match self.iter.next() {
			Some((i, got)) if got == c => {
				if c == '\n' {
					self.location.nl();
				} else {
					self.location.step();
				}
				Ok(i)
			}
			Some((_, got)) => self.error(format!("expected {}, got {}", c, got)).into(),
			None => self.error(format!("expected {}, got EOF", c)).into(),
		}
	}

	fn error(&self, reason: String) -> Error {
		Error::tokenizing_error(format!("({}) {}", self.location(), reason))
	}

	fn location(&self) -> String {
		format!("in {:#?} at {}", self.path, self.location)
	}
}

impl<'a> Iterator for Tokenizer<'a> {
	type Item = Fallible<ConfigToken<'a>>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.tokenize_next() {
			Ok(ConfigToken::Eof) => None,
			Ok(token) => Some(Ok(token)),
			Err(e) => Some(Err(e)),
		}
	}
}

pub fn parse_source<'source>(
	source: &'source str,
	path: OsString,
) -> impl IntoIterator<Item = Fallible<ConfigToken<'source>>> {
	Tokenizer::new(source, path)
}
