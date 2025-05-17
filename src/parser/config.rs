use std::collections::HashMap;
use std::fs::File;
use std::io;

use crate::material_newtype::MattyScheme;
use crate::parser::template::Template;

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
