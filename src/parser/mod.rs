mod common;
mod parser;
mod template;
mod tokenizer;

use std::{
    ffi::OsString,
    fs::File,
    io::{self, Read},
    path::Path,
};

use parser::{Config, parse_tokens};
use tokenizer::parse_source;

use crate::error::Fallible;

pub struct ConfigFile {
    path: OsString,
    source: String,
}

impl ConfigFile {
    pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        let mut file = File::open(path.as_ref())?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;

        Ok(ConfigFile {
            path: path.as_ref().to_path_buf().into_os_string(),
            source: buf,
        })
    }

    pub fn parse_config(&self) -> Fallible<Config<'_>> {
        let tokens = parse_source(&self.source, self.path.clone())?;

        parse_tokens(tokens, self.path.clone())
    }
}
