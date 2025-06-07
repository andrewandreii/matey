use std::{
	env::Args,
	iter::{Peekable, repeat_n},
};

pub enum ArgType {
	Flag,
	String,
}

pub struct Arg {
	name: &'static str,
	short: Option<&'static str>,
	long: Option<&'static str>,
	help: &'static str,
	type_: ArgType,
}

impl Arg {
	pub const fn new(
		name: &'static str,
		short: Option<&'static str>,
		long: Option<&'static str>,
		help: &'static str,
		type_: ArgType,
	) -> Self {
		if short.is_none() && long.is_none() {
			panic!("argument must either have a short name or a long name");
		}

		Arg {
			name,
			short: if let Some(short) = short {
				if short.len() == 2 {
					Some(short)
				} else {
					panic!("short name must be 2 characters")
				}
			} else {
				None
			},
			long,
			help,
			type_,
		}
	}

	pub fn matches(&self, arg: &str) -> bool {
		self.long.map_or(false, |e| e == arg) || self.short.map_or(false, |e| e == arg)
	}

	pub const fn short_len(&self) -> usize {
		2
	}

	pub fn long_len(&self) -> usize {
		self.long.map_or(0, |e| e.len())
	}
}

pub struct ArgParserBuilder {
	args: Args,
	opts: Vec<Arg>,
	last_opt: Arg,
	priority_opts: Vec<Arg>,
}

impl ArgParserBuilder {
	pub fn new(mut args: Args, last_opt: Arg) -> Self {
		args.next();
		ArgParserBuilder {
			last_opt,
			args,
			priority_opts: Vec::new(),
			opts: Vec::new(),
		}
	}

	pub fn add_opt(mut self, arg: Arg) -> Self {
		self.opts.push(arg);
		self
	}

	pub fn add_priority_opt(mut self, arg: Arg) -> Self {
		self.priority_opts.push(arg);
		self
	}

	pub fn build(self) -> ArgParser {
		ArgParser {
			args: self.args.peekable(),
			opts: self.opts,
			last_opt: self.last_opt,
			last_opt_read: false,
			priority_opts: self.priority_opts,
		}
	}
}

pub struct ArgParser {
	args: Peekable<Args>,
	opts: Vec<Arg>,
	last_opt: Arg,
	priority_opts: Vec<Arg>,
	last_opt_read: bool,
}

impl ArgParser {
	pub fn new(args: Args, opts: Vec<Arg>, priority_opts: Vec<Arg>, last_opt: Arg) -> Self {
		ArgParser {
			args: args.peekable(),
			opts,
			last_opt,
			last_opt_read: false,
			priority_opts,
		}
	}

	fn arg_help_len(arg: &Arg) -> usize {
		2 + arg.short_len() + 2 + arg.long_len() + 3
	}

	pub fn emit_help(&self) {
		let mut largest = 0;
		for arg in &self.opts {
			let current = ArgParser::arg_help_len(arg);

			if current > largest {
				largest = current;
			}
		}

		println!("Usage: matey [OPTIONS]... [-i] FILE");
		println!("Generate theme for FILE and write configs with given templates\n");

		for opt in self
			.opts
			.iter()
			.chain(self.priority_opts.iter())
			.chain([&self.last_opt])
		{
			match (opt.short, opt.long) {
				(Some(short), Some(long)) => print!("  {}, {}   ", short, long),
				(None, Some(long)) => print!("      {}   ", long),
				(Some(short), None) => print!("  {}     ", short),
				(None, None) => unreachable!(),
			}
			let count = largest - ArgParser::arg_help_len(opt);
			print!("{}", repeat_n(" ", count).collect::<String>());
			println!("{}", opt.help);
		}
	}
}

impl Iterator for ArgParser {
	type Item = (&'static str, Option<String>);

	fn next(&mut self) -> Option<Self::Item> {
		let with_next_arg = |arg: &Arg, got: &str, args: &mut Peekable<Args>| {
			if let Some(value) = args.next() {
				Some((arg.name, Some(value)))
			} else {
				panic!("{} needs a value", got);
			}
		};

		if let Some(next_arg) = self.args.next() {
			if let Some(arg) = self.priority_opts.iter().find(|arg| arg.matches(&next_arg)) {
				Some((arg.name, Some(next_arg)))
			} else if self.args.peek().is_none() && !self.last_opt_read {
				self.last_opt_read = true;
				Some((self.last_opt.name, Some(next_arg)))
			} else if self.last_opt.matches(&next_arg) {
				self.last_opt_read = true;
				with_next_arg(&self.last_opt, &next_arg, &mut self.args)
			} else if let Some(arg) = self.opts.iter().find(|arg| arg.matches(&next_arg)) {
				match arg.type_ {
					ArgType::Flag => Some((arg.name, None)),
					ArgType::String => with_next_arg(&arg, &next_arg, &mut self.args),
				}
			} else {
				panic!("unknown argument {next_arg}");
			}
		} else {
			if !self.last_opt_read {
				panic!("{} must be specified", self.last_opt.name);
			}
			None
		}
	}
}
