#![allow(clippy::uninlined_format_args)]

mod common;
mod config;
mod parser;
mod templates;
mod tokenizer;

use std::path::Path;

use config::Config;
use parser::parse_tokens;
use tokenizer::parse_source;

use crate::error::Fallible;

pub use templates::{CharIndex, IndexableVariable};

pub fn parse_config<'a>(path: impl AsRef<Path>, source: &'a str) -> Fallible<Config<'a>> {
	let tokens = parse_source(source, path.as_ref().into());

	parse_tokens(tokens, path.as_ref().into())
}

mod test {
	#[allow(unused_imports)]
	use crate::parsers::parse_config;

	#[test]
	fn test_template() {
		let templates = [
			"#out \"test.out\"\nforeach{{color}={name}}".to_string(),
			"#out outfile\nforeach{}norm{{image}}".to_string(),
		];
		for template in templates {
			let config = parse_config("test.path", &template);
			println!("{:?}", config);
			assert!(config.is_ok());
		}
	}
}
