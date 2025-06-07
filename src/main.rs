#![allow(clippy::uninlined_format_args)]

use std::error::Error;
use std::fs;
use std::path::{PathBuf, absolute};
use std::{env, fs::File, io::Read};

use log::{LevelFilter, error, info};
use material_colors::{image::ImageReader, theme::ThemeBuilder};

use matey::cache::Cacher;
use matey::material_newtype::MateyTheme;
use matey::parsers::IndexableVariable;
use matey::parsers::parse_config;
use simple_logger::SimpleLogger;

fn try_load_from_config(template_files: &mut Vec<PathBuf>) -> Result<PathBuf, Box<dyn Error>> {
	let mut config_path = PathBuf::new();

	if let Ok(path) = env::var("XDG_CONFIG_HOME") {
		config_path.push(path);
	} else {
		config_path.push(env::var("HOME")?);
		config_path.push(".config");
	}

	config_path.push("matey");
	fs::create_dir_all(&config_path)?;

	for entry in fs::read_dir(&config_path)? {
		template_files.push(entry?.path());
	}

	Ok(config_path)
}

fn compute_theme(buffer: &[u8]) -> MateyTheme {
	let mut image = ImageReader::read(buffer).expect("Could not parse image");
	image.resize(128, 128, material_colors::image::FilterType::Lanczos3);
	let theme = ThemeBuilder::with_source(ImageReader::extract_color(&image)).build();
	MateyTheme::new(theme.schemes.light.into(), theme.schemes.dark.into())
}

fn main() -> Result<(), Box<dyn Error>> {
	let mut args = env::args().skip(1).peekable();

	let mut template_files: Vec<PathBuf> = Vec::new();

	let mut image_path: Option<String> = None;
	let mut use_cache = false;
	let mut is_dark = true;
	let mut dry_run = false;
	let mut no_configs = false;
	let mut log_level = LevelFilter::Warn;
	while let Some(arg) = args.next() {
		match arg.as_str() {
			"-i" => {
				let path: PathBuf = args.next().expect("-i requires an argument").into();
				template_files.push(absolute(path)?);
			}
			"-f" => {
				image_path = Some(args.next().expect("-f requires an argument"));
			}
			"-u" | "--use-cache" => {
				use_cache = true;
			}
			"-l" => {
				is_dark = false;
			}
			"--no-configs" => {
				no_configs = true;
			}
			"--dry-run" => {
				dry_run = true;
			}
			"-q" | "--quiet" => {
				log_level = LevelFilter::Off;
			}
			"-v" | "--verbose" => {
				log_level = LevelFilter::Info;
			}
			other => {
				panic!("Unknown option {}", other);
			}
		}
	}

	SimpleLogger::new().with_level(log_level).init().unwrap();

	let config_path = if !no_configs {
		match try_load_from_config(&mut template_files) {
			Err(e) => {
				error!("could not load templates form config: {}", e);
				None
			}
			Ok(mut path) => {
				path.pop();
				Some(path)
			}
		}
	} else {
		None
	};

	let image_path = if let Some(file) = image_path {
		file
	} else {
		panic!("Please provide an image with -f")
	};

	let buffer = fs::read(&image_path).expect("Could not read image");

	let scheme = if use_cache {
		let cacher = Cacher::new("matey")?;
		let handle = cacher.get(&buffer);
		match cacher.get_cache(&handle) {
			Some(Ok(theme)) => theme,
			Some(Err(e)) => {
				error!("error loading cache: {}", e);
				compute_theme(&buffer)
			}
			None => {
				let theme = compute_theme(&buffer);

				if cacher.save_cache(&handle, &theme).is_err() {
					error!("could not save theme to cache");
				}

				theme
			}
		}
	} else {
		compute_theme(&buffer)
	};

	let theme = if is_dark { &scheme.dark } else { &scheme.light };

	let additional = [
		(
			"image".to_string(),
			IndexableVariable::plain(image_path.clone().into_bytes()),
		),
		(
			"HOME".to_string(),
			IndexableVariable::plain(env::var_os("HOME").unwrap_or_default().into_encoded_bytes()),
		),
		(
			"CONFIG".to_string(),
			IndexableVariable::plain(
				config_path
					.unwrap_or_default()
					.into_os_string()
					.into_encoded_bytes(),
			),
		),
	];

	let hashmap = (theme)
		.into_iter()
		.map::<(String, IndexableVariable), _>(|(key, color)| (key.to_string(), (*color).into()))
		.chain(additional)
		.collect();

	for path in template_files {
		info!("parsing {}", path.display());
		let mut file = match File::open(&path) {
			Ok(file) => file,
			Err(e) => {
				error!("while opening {}: {}", path.display(), e);
				continue;
			}
		};
		let mut buf = String::new();

		if let Err(e) = file.read_to_string(&mut buf) {
			error!("while reading {}: {}", path.display(), e);
			continue;
		}

		let config = match parse_config(&path, &buf) {
			Ok(config) => config,
			Err(e) => {
				error!("while parsing {}: {}", path.display(), e);
				continue;
			}
		};
		if !dry_run {
			if let Err(e) = config.write(theme, &hashmap) {
				error!("while writing template {}: {}", path.display(), e);
			}
		}
	}

	Ok(())
}
