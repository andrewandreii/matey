use std::ffi::OsString;
use std::iter::Peekable;

use crate::error::{Error, Fallible};
use crate::parsers::config::Config;
use crate::parsers::config::ConfigBuilder;

use super::templates::Template;
use super::tokenizer::{ConfigToken, Token};

fn parse_error(filename: &OsString, token: &Token, message: String) -> Error {
	Error::parse_error(format!(
		"({:#?} at {}) {}",
		filename, token.location, message
	))
}

fn expect_token<'a, 'b, I>(tokens: &mut Peekable<I>, token: ConfigToken<'b>) -> bool
where
	I: Iterator<Item = Fallible<ConfigToken<'a>>>,
{
	tokens
		.next_if(|tt| tt.is_ok() && *tt.as_ref().unwrap() == token)
		.is_some()
}

pub fn parse_tokens<'a, I>(tokens: I, filename: OsString) -> Fallible<Config<'a>>
where
	I: IntoIterator<Item = Fallible<ConfigToken<'a>>>,
{
	let mut iter = tokens.into_iter().peekable();

	let mut config_builder = ConfigBuilder::new("default.conf");
	while let Some(token) = iter.peek() {
		let token = match token {
			Ok(token) => *token,
			Err(e) => return Err(e.clone()),
		};

		match token {
			ConfigToken::OptionCommand(command) => {
				// FIXME: this is temporary
				iter.next();

				let mut is_template = false;
				let arg = match iter.next() {
					Some(Ok(ConfigToken::Literal(arg))) => arg.source,
					Some(Ok(ConfigToken::TemplateBlock(arg))) => {
						is_template = true;
						arg.source
					}
					t => {
						println!("got {:?}", t);
						return parse_error(
							&filename,
							&command,
							"expected literal after command token".to_string(),
						)
						.into();
					}
				};

				match command.source {
					"out" => {
						if is_template {
							config_builder.set_outfile_template(Template::new(arg));
						} else {
							config_builder.set_outfile(arg);
						}
					}
					"naming" => {
						config_builder.set_naming(arg);
					}
					unknown => {
						return parse_error(
							&filename,
							&command,
							format!("unknown command {}", unknown),
						)
						.into();
					}
				}

				if !expect_token(&mut iter, ConfigToken::Eof)
					&& !expect_token(&mut iter, ConfigToken::NewLine)
				{
					return parse_error(
						&filename,
						&command,
						format!("expected newline after command got {:?}", iter.next()),
					)
					.into();
				}
			}
			ConfigToken::Id(name) => {
				iter.next();

				let template = match iter.next() {
					Some(Ok(ConfigToken::TemplateBlock(template))) => template,
					Some(Err(e)) => return Err(e),
					None | Some(Ok(_)) => {
						return parse_error(
							&filename,
							&name,
							format!("expected template after id {}", name.source),
						)
						.into();
					}
				};

				match name.source {
					"foreach" => {
						config_builder.add_foreach_template(Template::new(template.source))
					}
					"norm" => config_builder.add_norm_template(Template::new(template.source)),
					_ => {
						return parse_error(
							&filename,
							&name,
							format!("unsupported template type {}", name.source),
						)
						.into();
					}
				}
			}
			ConfigToken::Number(_token) => todo!(),
			ConfigToken::TemplateBlock(token) => {
				return parse_error(
					&filename,
					&token,
					"Unnamed templates aren't allowed".to_string(),
				)
				.into();
			}
			ConfigToken::NewLine => {
				iter.next();
			}
			ConfigToken::Eof => {
				break;
			}
			ConfigToken::Literal(token) => {
				return parse_error(&filename, &token, "Unexpected string".to_string()).into();
			}
		}
	}

	Ok(config_builder.build())
}
