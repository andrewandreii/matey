use std::{collections::HashMap, ffi::OsString, fmt::Debug, fs::File, io};

use crate::error::{Error, Fallible};
use crate::material_newtype::MattyScheme;

use super::template::Template;
use super::tokenizer::{ConfigToken, Token};

#[derive(Debug)]
pub struct ConfigOptions<'a> {
    pub outfile: &'a str,
}

impl<'a> ConfigOptions<'a> {
    pub fn new(outfile: &'a str) -> Self {
        ConfigOptions { outfile }
    }
}

#[derive(Debug)]
pub struct Config<'a> {
    options: ConfigOptions<'a>,
    foreach_template: Template<'a>,
    additional_template: Template<'a>,
}

impl<'a> Config<'a> {
    pub fn write(&self, scheme: &MattyScheme, hashmap: &HashMap<String, String>) -> io::Result<()> {
        let mut file = File::create(self.options.outfile)?;

        self.foreach_template.run_with_scheme(&mut file, scheme)?;

        self.additional_template
            .run_with_hashmap(&mut file, &hashmap)?;

        Ok(())
    }
}

fn parse_error(filename: &OsString, token: &Token, message: String) -> Error {
    Error::parse_error(format!(
        "({:#?} at {}) {}",
        filename, token.location, message
    ))
}

pub fn parse_tokens<'a, I>(tokens: I, filename: OsString) -> Fallible<Config<'a>>
where
    I: IntoIterator<Item = Fallible<ConfigToken<'a>>>,
{
    let mut iter = tokens.into_iter().peekable();

    let mut options = ConfigOptions::new("default.conf");
    let mut foreach_template: Option<Template<'a>> = None;
    let mut additional_template: Option<Template<'a>> = None;
    while let Some(token) = iter.peek() {
        let token = match token {
            Ok(token) => *token,
            Err(e) => return Err(e.clone()),
        };

        match token {
            ConfigToken::OptionCommand(command) => match command.source {
                "out" => {
                    iter.next();
                    match iter.next() {
                        Some(Ok(ConfigToken::Id(outfile))) => options.outfile = outfile.source,
                        Some(Ok(ConfigToken::Literal(outfile))) => options.outfile = outfile.source,
                        _ => {
                            return parse_error(
                                &filename,
                                &command,
                                "expected id after command token".to_string(),
                            )
                            .into();
                        }
                    }

                    if iter
                        .next_if(|tt| tt.is_ok() && *tt.as_ref().unwrap() == ConfigToken::Eof)
                        .is_none()
                    {
                        if iter
                            .next_if(|tt| {
                                tt.is_ok() && *tt.as_ref().unwrap() == ConfigToken::NewLine
                            })
                            .is_none()
                        {
                            return parse_error(
                                &filename,
                                &command,
                                format!("expected newline after command got {:?}", iter.next()),
                            )
                            .into();
                        }
                    }
                }
                unknown => {
                    return parse_error(
                        &filename,
                        &command,
                        format!("unknown command {}", unknown),
                    )
                    .into();
                }
            },
            ConfigToken::Id(name) => {
                iter.next();
                let template_ref = match name.source {
                    "foreach" => &mut foreach_template,
                    "once" => &mut additional_template,
                    _ => {
                        return parse_error(
                            &filename,
                            &name,
                            format!("unsupported template type {}", name.source),
                        )
                        .into();
                    }
                };

                let next = match iter.next() {
                    Some(Ok(next)) => next,
                    Some(Err(e)) => return Err(e),
                    None => {
                        return parse_error(
                            &filename,
                            &name,
                            format!("expected template after id {}", name.source),
                        )
                        .into();
                    }
                };

                match next {
                    ConfigToken::TemplateBlock(template) => {
                        *template_ref = Some(Template::new(template.source));
                    }
                    _ => {
                        return parse_error(
                            &filename,
                            &name,
                            "expected template after id".to_string(),
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

    Ok(Config {
        options,
        foreach_template: foreach_template.unwrap_or(Template::new("")),
        additional_template: additional_template.unwrap_or(Template::new("")),
    })
}
