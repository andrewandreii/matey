use std::fmt;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct FileLocation {
	line: usize,
	column: usize,
}

impl FileLocation {
	pub fn new() -> Self {
		FileLocation { line: 1, column: 1 }
	}

	pub fn step(&mut self) {
		self.column += 1;
	}

	pub fn nl(&mut self) {
		self.column = 1;
		self.line += 1;
	}
}

impl fmt::Display for FileLocation {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "line: {}, col: {}", self.line, self.column)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenamingScheme {
	Snake,
	Dash,
	Camel,
	Upper,
	UpperSnake,
	UpperCamel,
	Flat,
}

fn from_snake(s: &str, make_first_upper: bool) -> String {
	if s.is_empty() {
		return String::new();
	}

	let mut fin = String::with_capacity(s.len());

	let mut chunks = s.split('_').filter(|s| !s.is_empty());
	if !make_first_upper {
		if let Some(s) = chunks.next() {
			fin.push_str(s);
		}
	}

	for chunk in chunks {
		fin.push(chunk.chars().next().unwrap().to_ascii_uppercase());
		fin.push_str(&chunk[1..]);
	}

	fin
}

pub fn rename_from_snake_case<S: AsRef<str>>(s: S, renaming: RenamingScheme) -> String {
	use RenamingScheme::*;
	match renaming {
		Dash => s.as_ref().replace("_", "-"),
		Snake => s.as_ref().to_owned(),
		Upper => s
			.as_ref()
			.chars()
			.filter_map(|c| {
				if c == '_' {
					None
				} else {
					Some(c.to_ascii_uppercase())
				}
			})
			.collect(),
		UpperSnake => s.as_ref().to_uppercase(),
		Flat => s.as_ref().replace("_", ""),
		Camel => from_snake(s.as_ref(), false),
		UpperCamel => from_snake(s.as_ref(), true),
	}
}
