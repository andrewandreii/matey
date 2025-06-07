use std::collections::HashMap;
use std::io;
use std::iter::Peekable;
use std::vec::Vec;

use log::warn;

use super::indexable::{CharIndex, IndexableVariable};

use crate::material_newtype::MateyScheme;
use crate::parsers::common::{RenamingScheme, rename_from_snake_case};

#[derive(Debug)]
pub enum TemplateToken<'a> {
	RawString(&'a str),
	Key(&'a str),
	IndexedKey(&'a str, &'a str),
}

#[derive(Debug)]
pub struct Template<'a> {
	expr: Vec<TemplateToken<'a>>,
}

fn peekable_next_until<T: Iterator>(
	iter: &mut Peekable<T>,
	mut cond: impl FnMut(&T::Item) -> bool,
) -> T::Item {
	let mut opt_item = iter.next();
	while let Some(item) = opt_item {
		match iter.peek() {
			Some(next) => {
				if cond(next) {
					return item;
				}
			}
			None => {
				return item;
			}
		}
		opt_item = iter.next();
	}

	opt_item.unwrap()
}

impl<'a> Template<'a> {
	pub fn new(template: &'a str) -> Self {
		let mut tokens = Vec::new();
		let mut iter = template.char_indices().peekable();
		let mut escaped = false;
		while let Some((i, c)) = iter.peek() {
			match c {
				'{' if !escaped => {
					let start = *i + 1;
					let (end, _) = peekable_next_until(&mut iter, |(_, c)| *c == '}');
					iter.next();
					let whole = &template[start..=end].trim();
					if let Some(idx) = whole.find('.') {
						tokens.push(TemplateToken::IndexedKey(&whole[..idx], &whole[idx + 1..]));
					} else {
						tokens.push(TemplateToken::Key(whole));
					}
				}
				'\\' => {
					escaped = true;
					iter.next();
				}
				_ => {
					let start = *i;

					let (end, _) = peekable_next_until(&mut iter, |(_, c)| *c == '{' || *c == '\\');
					escaped = false;

					tokens.push(TemplateToken::RawString(&template[start..=end]));
				}
			}
		}

		Template { expr: tokens }
	}

	pub fn run_with_hashmap<W>(
		&self,
		writer: &mut W,
		hashmap: &HashMap<String, IndexableVariable>,
	) -> io::Result<()>
	where
		W: io::Write,
	{
		for token in &self.expr {
			match token {
				TemplateToken::RawString(s) => {
					writer.write_all(s.as_bytes())?;
				}
				TemplateToken::Key(key) => {
					if let Some(value) = hashmap.get(*key) {
						writer.write_all(&value.get_all())?;
					} else {
						warn!("key \"{key}\" not found");
					}
				}
				TemplateToken::IndexedKey(key, indexes) => {
					if let Some(value) = hashmap.get(*key) {
						write_indexed(writer, value, indexes, key, true)?;
					} else {
						warn!("warning: key \"{key}\" not found");
					}
				}
			}
		}

		Ok(())
	}

	pub fn run_with_scheme<W>(
		&self,
		writer: &mut W,
		scheme: &MateyScheme,
		rename: RenamingScheme,
	) -> io::Result<()>
	where
		W: io::Write,
	{
		let mut should_warn = true;

		for (name, color) in scheme {
			for token in &self.expr {
				match token {
					TemplateToken::RawString(s) => {
						writer.write_all(s.as_bytes())?;
					}
					TemplateToken::Key(key) => match *key {
						"name" => {
							writer.write_all(rename_from_snake_case(name, rename).as_bytes())?;
						}
						"color" => {
							writer.write_all(color.to_hex().as_bytes())?;
						}
						key => {
							if should_warn {
								warn!("unknown key \"{key}\" in foreach");
							}
						}
					},
					TemplateToken::IndexedKey(key, indexes) => match *key {
						"color" => {
							write_indexed(writer, color, indexes, key, should_warn)?;
						}
						key => {
							if should_warn {
								warn!("key \"{key}\" cannot be indexed");
							}
						}
					},
				}
			}

			should_warn = false;
		}

		Ok(())
	}
}

fn write_indexed<W, I>(
	writer: &mut W,
	value: I,
	indexes: &str,
	key: &str,
	should_warn: bool,
) -> io::Result<()>
where
	W: io::Write,
	I: CharIndex<ElementType = Vec<u8>>,
{
	for index in indexes.chars() {
		if let Some(v) = value.get(index) {
			writer.write_all(&v)?;
		} else if should_warn {
			warn!("warning: index \"{index}\" for key \"{key}\" not found");
		}
	}

	Ok(())
}
