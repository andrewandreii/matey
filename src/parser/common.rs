use std::fmt;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct FileLocation {
    line: usize,
    column: usize,
}

impl FileLocation {
    pub fn new(line: usize, column: usize) -> Self {
        FileLocation { line, column }
    }

    pub fn step(&mut self) {
        self.column += 1;
    }

    pub fn nl(&mut self) {
        self.column = 0;
        self.line += 1;
    }
}

impl fmt::Display for FileLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line: {}, col: {}", self.line, self.column)
    }
}
