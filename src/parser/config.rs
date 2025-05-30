use std::collections::HashMap;
use std::fs::File;

use super::common::RenamingScheme;

use crate::error::Error;
use crate::error::Fallible;
use crate::material_newtype::MattyScheme;
use crate::parser::template::IndexableVariable;
use crate::parser::template::Template;

#[derive(Debug)]
enum TemplatedString<'a> {
	Yes(Template<'a>),
	No(&'a str),
}

pub struct ConfigBuilder<'a> {
	outfile: TemplatedString<'a>,
	naming: RenamingScheme,
	templates: Vec<ConfigTemplate<'a>>,
}

impl<'a> ConfigBuilder<'a> {
	pub fn new<'b: 'a>(outfile: &'b str) -> Self {
		ConfigBuilder {
			outfile: TemplatedString::No(outfile),
			naming: RenamingScheme::SnakeCase,
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
		self.outfile = TemplatedString::No(outfile);
	}

	pub fn set_outfile_template<'b: 'a>(&mut self, outfile: Template<'b>) {
		self.outfile = TemplatedString::Yes(outfile);
	}

	pub fn set_naming<'b: 'a>(&mut self, naming: &'b str) {
		use RenamingScheme::*;
		self.naming = match naming {
			"snake_case" => SnakeCase,
			"UPPERCASE" => UpperCase,
			"UPPER_CASE" | "UPPER_SNAKE_CASE" => UpperSnakeCase,
			"dash-case" => DashCase,
			"camelCase" => CamelCase,
			"CamelCase" | "UpperCamelCase" => UpperCamelCase,
			"lowercase" | "flatcase" => FlatCase,
			_ => {
				println!("warning: unknown naming convention");
				SnakeCase
			}
		};
	}

	pub fn build(self) -> Config<'a> {
		Config {
			outfile: self.outfile,
			rename: self.naming,
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
	outfile: TemplatedString<'a>,
	rename: RenamingScheme,
	templates: Vec<ConfigTemplate<'a>>,
}

impl<'a> Config<'a> {
	pub fn write(
		&self,
		scheme: &MattyScheme,
		hashmap: &HashMap<String, IndexableVariable>,
	) -> Fallible<()> {
		let mut file = match &self.outfile {
			TemplatedString::Yes(template) => {
				let mut path = Vec::new();
				template
					.run_with_hashmap(&mut path, hashmap)
					.map_err(|e| Error::from_io(e))?;
				File::create(String::from_utf8(path.clone()).expect("Invalid file path. Aborting"))
					.map_err(|_| {
						Error::IOError(format!("Could not open file {:?}", unsafe {
							String::from_utf8_unchecked(path)
						}))
					})?
			}
			TemplatedString::No(path) => File::create(path)
				.map_err(|_| Error::IOError(format!("Could not open file {:?}", path)))?,
		};

		for template in &self.templates {
			match template {
				ConfigTemplate::Norm(template) => {
					template
						.run_with_hashmap(&mut file, hashmap)
						.map_err(|e| Error::from_io(e))?;
				}
				ConfigTemplate::Foreach(template) => {
					template
						.run_with_scheme(&mut file, scheme, self.rename)
						.map_err(|e| Error::from_io(e))?;
				}
			}
		}

		Ok(())
	}
}
