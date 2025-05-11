use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsString;
use std::fmt::LowerHex;
use std::path::{PathBuf, absolute};
use std::{env, fs::File, io::Read};
use std::{fmt, fs};

use material_colors::scheme::Scheme;
use material_colors::{image::ImageReader, theme::ThemeBuilder};

use matty::parser::ConfigFile;
use sha2::{Digest, Sha256};

use matty::material_newtype::MattyScheme;

struct HexSlice<'a>(&'a [u8]);

impl<'a> LowerHex for HexSlice<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in self.0 {
            write!(f, "{:x}", b)?;
        }

        Ok(())
    }
}

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

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1).peekable();

    let mut template_files: Vec<PathBuf> = Vec::new();

    if let Err(e) = try_load_from_config(&mut template_files) {
        println!("could not load templates form config: {}", e);
    }

    let mut file_string: Option<String> = None;
    let mut cache_image = false;
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
                file_string = Some(args.next().expect("-f requires an argument"));
            }
            "-c" | "--cache-image" => {
                cache_image = true;
            }
            "-u" | "--use-cache" => {
                use_cache = true;
            }
            "--uc" => {
                use_cache = true;
                cache_image = true;
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

    let file_string = if let Some(file) = file_string {
        file
    } else {
        panic!("Please provide an image with -f");
    };

    let mut file = File::open(&file_string).expect("Could not open image file");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .expect("Could not read image file");

    let mut cache_folder = PathBuf::new();
    let mut cache_img = PathBuf::new();
    if use_cache || cache_image {
        let home = env::var("HOME").expect("HOME environmental variable not found");
        cache_folder.push(home);
        cache_folder.push(".cache/matty");
        fs::create_dir_all(&cache_folder).expect("Could not create cache folder");

        let digest = Some(Sha256::digest(&buffer));
        cache_img = cache_folder.clone();
        cache_img.push(format!("{:x}", HexSlice(digest.unwrap().as_slice())));
    }

    let compute_theme = || -> Scheme {
        let mut image = ImageReader::read(&buffer).expect("Could not parse image");
        image.resize(128, 128, material_colors::image::FilterType::Lanczos3);
        let theme = ThemeBuilder::with_source(ImageReader::extract_color(&image)).build();
        if is_dark {
            theme.schemes.dark
        } else {
            theme.schemes.light
        }
    };

    let scheme: MattyScheme = match (use_cache, File::open(&cache_img)) {
        (false, _) => compute_theme().into(),
        (true, Ok(file)) => serde_json::from_reader(file)?,
        (true, Err(_)) => {
            use_cache = false;
            if !cache_image {
                println!("warning: cache for image not found, run with -c to generate");
            }
            compute_theme().into()
        }
    };

    if cache_image && !use_cache {
        let cache_file = File::create(cache_img).expect("Could not create cache file");
        serde_json::to_writer(cache_file, &scheme).expect("Could not write cache");
    }

    cache_folder.push("output");
    fs::create_dir_all(&cache_folder).expect("Could not create output folder");
    env::set_current_dir(cache_folder).expect("Could not change directory to cache folder");

    let additional = [("image".to_string(), file_string.clone())];
    let hashmap = (&scheme)
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
            config.write(&scheme, &hashmap)?;
        }
    }

    Ok(())
}
