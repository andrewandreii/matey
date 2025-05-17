use std::{collections::HashMap, ffi::OsString, fmt::Debug, fs::File, io};

use crate::error::{Error, Fallible};
use crate::material_newtype::MattyScheme;

use super::template::Template;
use super::tokenizer::{ConfigToken, Token};

pub struct ConfigBuilder<'a> {
    outfile: &'a str,
    templates: Vec<ConfigTemplate<'a>>,
}

impl<'a> ConfigBuilder<'a> {
    pub fn new<'b: 'a>(outfile: &'b str) -> Self {
        ConfigBuilder {
            outfile,
            templates: Vec::new(),
        }
    }

    pub fn add_foreach_template<'b: 'a>(&mut self, template: Template<'b>) {
        self.templates.push(ConfigTemplate::Foreach(template));
    }

    pub fn add_norm_template<'b: 'a>(&mut self, template: Template<'b>) {
        self.templates.push(ConfigTemplate::Norm(template));
    }

    pub fn set_outfile<'b: 'a>(&mut self, outfile: &'b str) {
        self.outfile = outfile;
    }

    pub fn build(self) -> Config<'a> {
        Config {
            outfile: self.outfile,
            templates: self.templates,
        }
    }
}

#[derive(Debug)]
enum ConfigTemplate<'a> {
    Foreach(Template<'a>),
    Norm(Template<'a>),
}

#[derive(Debug)]
pub struct Config<'a> {
    outfile: &'a str,
    templates: Vec<ConfigTemplate<'a>>,
}

impl<'a> Config<'a> {
    pub fn write(&self, scheme: &MattyScheme, hashmap: &HashMap<String, String>) -> io::Result<()> {
        let mut file = File::create(self.outfile)?;

        for template in &self.templates {
            match template {
                ConfigTemplate::Norm(template) => {
                    template.run_with_hashmap(&mut file, hashmap)?;
                }
                ConfigTemplate::Foreach(template) => {
                    template.run_with_scheme(&mut file, scheme)?;
                }
            }
        }

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

    let mut config_builder = ConfigBuilder::new("default.conf");
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
                        Some(Ok(ConfigToken::Id(outfile))) => {
                            config_builder.set_outfile(outfile.source)
                        }
                        Some(Ok(ConfigToken::Literal(outfile))) => {
                            config_builder.set_outfile(outfile.source)
                        }
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
