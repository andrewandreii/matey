use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsString;
use std::fmt::LowerHex;
use std::path::PathBuf;
use std::{env, fs::File, io::Read};
use std::{fmt, fs, ptr};

use material_colors::scheme::Scheme;
use material_colors::{color::Argb, image::ImageReader, theme::ThemeBuilder};

use matty::parser::ConfigFile;
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Deserialize)]
struct Pain {
    pub primary: Why,
    pub on_primary: Why,
    pub primary_container: Why,
    pub on_primary_container: Why,
    pub inverse_primary: Why,
    pub primary_fixed: Why,
    pub primary_fixed_dim: Why,
    pub on_primary_fixed: Why,
    pub on_primary_fixed_variant: Why,
    pub secondary: Why,
    pub on_secondary: Why,
    pub secondary_container: Why,
    pub on_secondary_container: Why,
    pub secondary_fixed: Why,
    pub secondary_fixed_dim: Why,
    pub on_secondary_fixed: Why,
    pub on_secondary_fixed_variant: Why,
    pub tertiary: Why,
    pub on_tertiary: Why,
    pub tertiary_container: Why,
    pub on_tertiary_container: Why,
    pub tertiary_fixed: Why,
    pub tertiary_fixed_dim: Why,
    pub on_tertiary_fixed: Why,
    pub on_tertiary_fixed_variant: Why,
    pub error: Why,
    pub on_error: Why,
    pub error_container: Why,
    pub on_error_container: Why,
    pub surface_dim: Why,
    pub surface: Why,
    pub surface_tint: Why,
    pub surface_bright: Why,
    pub surface_container_lowest: Why,
    pub surface_container_low: Why,
    pub surface_container: Why,
    pub surface_container_high: Why,
    pub surface_container_highest: Why,
    pub on_surface: Why,
    pub on_surface_variant: Why,
    pub outline: Why,
    pub outline_variant: Why,
    pub inverse_surface: Why,
    pub inverse_on_surface: Why,
    pub surface_variant: Why,
    pub background: Why,
    pub on_background: Why,
    pub shadow: Why,
    pub scrim: Why,
}

impl From<Pain> for Scheme {
    fn from(value: Pain) -> Self {
        Scheme {
            primary: value.primary.into(),
            on_primary: value.on_primary.into(),
            primary_container: value.primary_container.into(),
            on_primary_container: value.on_primary_container.into(),
            inverse_primary: value.inverse_primary.into(),
            primary_fixed: value.primary_fixed.into(),
            primary_fixed_dim: value.primary_fixed_dim.into(),
            on_primary_fixed: value.on_primary_fixed.into(),
            on_primary_fixed_variant: value.on_primary_fixed_variant.into(),
            secondary: value.secondary.into(),
            on_secondary: value.on_secondary.into(),
            secondary_container: value.secondary_container.into(),
            on_secondary_container: value.on_secondary_container.into(),
            secondary_fixed: value.secondary_fixed.into(),
            secondary_fixed_dim: value.secondary_fixed_dim.into(),
            on_secondary_fixed: value.on_secondary_fixed.into(),
            on_secondary_fixed_variant: value.on_secondary_fixed_variant.into(),
            tertiary: value.tertiary.into(),
            on_tertiary: value.on_tertiary.into(),
            tertiary_container: value.tertiary_container.into(),
            on_tertiary_container: value.on_tertiary_container.into(),
            tertiary_fixed: value.tertiary_fixed.into(),
            tertiary_fixed_dim: value.tertiary_fixed_dim.into(),
            on_tertiary_fixed: value.on_tertiary_fixed.into(),
            on_tertiary_fixed_variant: value.on_tertiary_fixed_variant.into(),
            error: value.error.into(),
            on_error: value.on_error.into(),
            error_container: value.error_container.into(),
            on_error_container: value.on_error_container.into(),
            surface_dim: value.surface_dim.into(),
            surface: value.surface.into(),
            surface_tint: value.surface_tint.into(),
            surface_bright: value.surface_bright.into(),
            surface_container_lowest: value.surface_container_lowest.into(),
            surface_container_low: value.surface_container_low.into(),
            surface_container: value.surface_container.into(),
            surface_container_high: value.surface_container_high.into(),
            surface_container_highest: value.surface_container_highest.into(),
            on_surface: value.on_surface.into(),
            on_surface_variant: value.on_surface_variant.into(),
            outline: value.outline.into(),
            outline_variant: value.outline_variant.into(),
            inverse_surface: value.inverse_surface.into(),
            inverse_on_surface: value.inverse_on_surface.into(),
            surface_variant: value.surface_variant.into(),
            background: value.background.into(),
            on_background: value.on_background.into(),
            shadow: value.shadow.into(),
            scrim: value.scrim.into(),
        }
    }
}

#[derive(Deserialize)]
struct Why {
    pub alpha: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl From<Why> for Argb {
    fn from(value: Why) -> Self {
        Argb {
            alpha: value.alpha,
            red: value.red,
            green: value.green,
            blue: value.blue,
        }
    }
}

fn deserialize_scheme(file: File) -> Result<Scheme, Box<dyn Error>> {
    let why_not_just_implement_deserialize = serde_json::from_reader::<_, Pain>(file)?;
    Ok(why_not_just_implement_deserialize.into())
}

struct HexSlice<'a>(&'a [u8]);

impl<'a> LowerHex for HexSlice<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in self.0 {
            write!(f, "{:x}", b)?;
        }

        Ok(())
    }
}

fn try_load_from_config(template_files: &mut Vec<OsString>) -> Result<(), Box<dyn Error>> {
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
        template_files.push(entry?.path().into_os_string());
    }

    Ok(())
}

fn clone_scheme(scheme: &Scheme) -> Scheme {
    // SAFETY: Scheme is just a lot of u8 put together, ok to copy
    unsafe { ptr::read(scheme) }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1).peekable();

    let mut template_files: Vec<OsString> = Vec::new();

    if let Err(e) = try_load_from_config(&mut template_files) {
        println!("could not load templates form config: {}", e);
    }

    let mut file_string: Option<String> = None;
    let mut cache_image = false;
    let mut use_cache = false;
    let mut is_dark = true;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-i" => {
                template_files.push(args.next().expect("-i requires an argument").into());
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

    let mut file = File::open(&file_string)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let mut cache_folder = PathBuf::new();
    let mut cache_img = PathBuf::new();
    if use_cache || cache_image {
        let home = env::var("HOME")?;
        cache_folder.push(home);
        cache_folder.push(".cache/matty");
        fs::create_dir_all(&cache_folder)?;

        let digest = Some(Sha256::digest(&buffer));
        cache_img = cache_folder.clone();
        cache_img.push(format!("{:x}", HexSlice(digest.unwrap().as_slice())));
    }

    let compute_theme = || -> Result<Scheme, Box<dyn Error>> {
        let mut image = ImageReader::read(&buffer)?;
        image.resize(128, 128, material_colors::image::FilterType::Lanczos3);
        let theme = ThemeBuilder::with_source(ImageReader::extract_color(&image)).build();
        Ok(if is_dark {
            theme.schemes.dark
        } else {
            theme.schemes.light
        })
    };

    let scheme = match (use_cache, File::open(&cache_img)) {
        (false, _) => compute_theme()?,
        (true, Ok(file)) => deserialize_scheme(file)?,
        (true, Err(_)) => {
            use_cache = false;
            if !cache_image {
                println!("warning: cache for image not found, run with -c to generate");
            }
            compute_theme()?
        }
    };

    if cache_image && !use_cache {
        let cache_file = File::create(cache_img)?;
        serde_json::to_writer(cache_file, &scheme)?;
    }

    cache_folder.push("output");
    fs::create_dir_all(&cache_folder)?;
    env::set_current_dir(cache_folder)?;

    let additional = [("image".to_string(), file_string.clone())];
    let hashmap = clone_scheme(&scheme)
        .into_iter()
        .map(|(key, color)| (key, color.to_hex()))
        .chain(additional)
        .collect::<HashMap<String, String>>();

    for file in template_files {
        let config = ConfigFile::new(&file)?;
        let config = config.parse_config()?;
        config.write(clone_scheme(&scheme), &hashmap)?;
    }

    Ok(())
}
