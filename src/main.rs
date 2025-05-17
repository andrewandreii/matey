use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{PathBuf, absolute};
use std::{env, fs::File, io::Read};

use material_colors::{image::ImageReader, theme::ThemeBuilder};

use matty::cache::Cacher;
use matty::parser::ConfigFile;

use matty::material_newtype::MattyTheme;

fn try_load_from_config(template_files: &mut Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
    let mut config_path = PathBuf::new();

    if let Ok(path) = env::var("XDG_CONFIG_HOME") {
        config_path.push(path);
    } else {
        config_path.push(env::var("HOME")?);
        config_path.push(".config");
    }

    config_path.push("matty");
    fs::create_dir_all(&config_path)?;

    for entry in fs::read_dir(config_path)? {
        template_files.push(entry?.path());
    }

    Ok(())
}

fn compute_theme(buffer: &[u8]) -> MattyTheme {
    let mut image = ImageReader::read(buffer).expect("Could not parse image");
    image.resize(128, 128, material_colors::image::FilterType::Lanczos3);
    let theme = ThemeBuilder::with_source(ImageReader::extract_color(&image)).build();
    MattyTheme::new(theme.schemes.light.into(), theme.schemes.dark.into())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1).peekable();

    let mut template_files: Vec<PathBuf> = Vec::new();

    let mut image_path: Option<String> = None;
    let mut use_cache = false;
    let mut is_dark = true;
    let mut dry_run = false;
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
            "--dry-run" => {
                dry_run = true;
            }
            other => {
                panic!("Unknown option {}", other);
            }
        }
    }

    if let Err(e) = try_load_from_config(&mut template_files) {
        println!("could not load templates form config: {}", e);
    }

    let image_path = if let Some(file) = image_path {
        file
    } else {
        panic!("Please provide an image with -f")
    };

    let buffer = fs::read(&image_path).expect("Could not read image");

    let scheme = if use_cache {
        let cacher = Cacher::new("matty")?;
        let handle = cacher.get(&buffer);
        match cacher.get_cache(&handle) {
            Some(Ok(theme)) => theme,
            Some(Err(e)) => {
                println!("error loading cache {e}");
                compute_theme(&buffer)
            }
            None => {
                let theme = compute_theme(&buffer);

                if cacher.save_cache(&handle, &theme).is_err() {
                    println!("Could not save theme to cache");
                }

                theme
            }
        }
    } else {
        compute_theme(&buffer)
    };

    let theme = if is_dark { &scheme.dark } else { &scheme.light };

    let additional = [("image".to_string(), image_path.clone())];
    let hashmap = (theme)
        .into_iter()
        .map(|(key, color)| (key.to_string(), color.to_hex()))
        .chain(additional)
        .collect::<HashMap<String, String>>();

    for path in template_files {
        let mut file = File::open(&path).expect("Could not open template file");
        let mut buf = String::new();
        file.read_to_string(&mut buf)
            .expect("Could not read template file");
        let config = ConfigFile::new(&path, buf);
        let config = config.parse_config()?;
        if !dry_run {
            config.write(theme, &hashmap)?;
        }
    }

    Ok(())
}
