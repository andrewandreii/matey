use std::collections::HashMap;
use std::fs::File;
use std::io;

use crate::material_newtype::MattyScheme;
use crate::parser::template::Template;

use super::common::RenamingScheme;

pub struct ConfigBuilder<'a> {
	outfile: &'a str,
	naming: RenamingScheme,
	templates: Vec<ConfigTemplate<'a>>,
}

impl<'a> ConfigBuilder<'a> {
	pub fn new<'b: 'a>(outfile: &'b str) -> Self {
		ConfigBuilder {
			outfile,
			naming: RenamingScheme::DashCase,
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
				DashCase
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
	outfile: &'a str,
	rename: RenamingScheme,
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
					template.run_with_scheme(&mut file, scheme, self.rename)?;
				}
			}
		}

		Ok(())
	}
}
