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
    pub fn new(path: impl AsRef<Path>, source: String) -> Self {
        ConfigFile {
            path: path.as_ref().to_path_buf().into_os_string(),
            source,
        }
    }

    pub fn parse_config(&self) -> Fallible<Config<'_>> {
        let tokens = parse_source(&self.source, self.path.clone())?;

        parse_tokens(tokens, self.path.clone())
    }
}

mod test {
    #[allow(unused_imports)]
    use super::ConfigFile;

    #[test]
    fn test_template() {
        let templates = [
            "#out \"test.out\"\nforeach{{color}={name}}".to_string(),
            "#out outfile\nforeach{}once{{image}}".to_string(),
        ];
        for template in templates {
            let cfile = ConfigFile::new("test.path", template);
            let config = cfile.parse_config();
            println!("{:?}", config);
            assert!(config.is_ok());
        }
    }
}
