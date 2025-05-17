use std::{
    env,
    fmt::{self, LowerHex},
    fs::{self, File},
    io::{ErrorKind, Read, Write},
    mem,
    path::{Path, PathBuf},
    slice,
};

use sha2::{Digest, Sha256};

use crate::{
    error::{Error, Fallible},
    material_newtype::MattyTheme,
};

pub struct Cacher {
    cache_folder: PathBuf,
}

impl Cacher {
    pub fn new(name: impl AsRef<Path>) -> Fallible<Cacher> {
        let mut cache_folder = if let Some(home) = env::var_os("HOME") {
            PathBuf::from(home)
        } else {
            let tmp = PathBuf::from("/tmp/");
            if tmp.is_dir() {
                tmp
            } else {
                return Error::IOError("cannot choose a folder for cache".to_string()).into();
            }
        };

        cache_folder.push(name);

        fs::create_dir_all(&cache_folder).map_err(|e| Error::IOError(e.to_string()))?;

        Ok(Cacher { cache_folder })
    }

    pub fn get(&self, raw: &[u8]) -> CacheHandle {
        let mut path = self.cache_folder.clone();
        path.push(format!("{:x}", HexSlice(Sha256::digest(raw).as_slice())));
        CacheHandle(path)
    }

    pub fn save_cache(&self, handle: &CacheHandle, theme: &MattyTheme) -> Fallible<()> {
        let mut file = File::create(handle.as_path())
            .map_err(|_| Error::IOError("Could not create cache file".to_string()))?;

        let buf = theme as *const MattyTheme as *const u8;
        let buf = unsafe { slice::from_raw_parts(buf, mem::size_of::<MattyTheme>()) };

        file.write_all(buf)
            .map_err(|_| Error::IOError("Could not write cache".to_string()))?;

        Ok(())
    }

    pub fn get_cache(&self, handle: &CacheHandle) -> Option<Fallible<MattyTheme>> {
        let mut cache = [0u8; mem::size_of::<MattyTheme>()];
        let mut file = match File::open(handle.as_path()) {
            Ok(file) => file,
            Err(e) if e.kind() == ErrorKind::NotFound => return None,
            Err(_) => return Some(Err(Error::IOError("Could not open file".to_string()))),
        };

        if let Err(e) = file.read_exact(&mut cache) {
            if e.kind() == ErrorKind::UnexpectedEof {
                return Some(Err(Error::IOError("Malformed cache".to_string())));
            }
            return Some(Err(Error::IOError("Could not read cache".to_string())));
        }

        Some(Ok(unsafe { mem::transmute(cache) }))
    }
}

#[derive(Debug, Clone)]
pub struct CacheHandle(PathBuf);
impl CacheHandle {
    fn as_path(&self) -> &PathBuf {
        &self.0
    }
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
