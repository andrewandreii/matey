use std::collections::HashMap;
use std::io;
use std::iter::Peekable;
use std::vec::Vec;

use crate::material_newtype::MattyScheme;

#[derive(Debug)]
pub enum TemplateToken<'a> {
    RawString(&'a str),
    Key(&'a str),
}

#[derive(Debug)]
pub struct Template<'a> {
    expr: Vec<TemplateToken<'a>>,
}

fn peekable_next_until<T: Iterator>(
    iter: &mut Peekable<T>,
    mut cond: impl FnMut(&T::Item) -> bool,
) -> T::Item {
    let mut opt_item = iter.next();
    while let Some(item) = opt_item {
        match iter.peek() {
            Some(next) => {
                if cond(next) {
                    return item;
                }
            }
            None => {
                return item;
            }
        }
        opt_item = iter.next();
    }

    return opt_item.unwrap();
}

impl<'a> Template<'a> {
    pub fn new(template: &'a str) -> Self {
        let mut tokens = Vec::new();
        let mut iter = template.char_indices().peekable();
        let mut escaped = false;
        while let Some((i, c)) = iter.peek() {
            match c {
                '{' if !escaped => {
                    let start = *i + 1;
                    let (end, _) = peekable_next_until(&mut iter, |(_, c)| *c == '}');
                    iter.next();
                    tokens.push(TemplateToken::Key(template[start..=end].trim()));
                }
                '\\' => {
                    escaped = true;
                    iter.next();
                }
                _ => {
                    let start = *i;

                    let (end, _) = peekable_next_until(&mut iter, |(_, c)| {
                        return *c == '{' || *c == '\\';
                    });
                    escaped = false;

                    tokens.push(TemplateToken::RawString(&template[start..=end]));
                }
            }
        }

        Template { expr: tokens }
    }

    pub fn run_with_hashmap<W>(
        &self,
        writer: &mut W,
        hashmap: &HashMap<String, String>,
    ) -> io::Result<()>
    where
        W: io::Write,
    {
        for token in &self.expr {
            match token {
                TemplateToken::RawString(s) => {
                    writer.write(s.as_bytes())?;
                }
                TemplateToken::Key(key) => {
                    if let Some(value) = hashmap.get(*key) {
                        writer.write(value.as_bytes())?;
                    } else {
                        println!("warning: key not found {}", key);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn run_with_scheme<W>(&self, writer: &mut W, scheme: &MattyScheme) -> io::Result<()>
    where
        W: io::Write,
    {
        for (name, color) in scheme {
            for token in &self.expr {
                match token {
                    TemplateToken::RawString(s) => {
                        writer.write(s.as_bytes())?;
                    }
                    TemplateToken::Key(key) => match *key {
                        "name" => {
                            writer.write(name.as_bytes())?;
                        }
                        "color" => {
                            writer.write(color.to_hex().as_bytes())?;
                        }
                        key => {
                            println!("warning: unknown key in template {}", key);
                        }
                    },
                }
            }
        }

        Ok(())
    }
}
