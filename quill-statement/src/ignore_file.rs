//! Read and parse the ignore files written by the user.

use serde::Deserialize;
use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};
use toml::value::Datetime;

use super::IGNOREFILE;

/// An intermediate format for parsing ignore files.
/// This intermediate exists to simplify deserialization with TOML.
#[derive(Debug, Deserialize)]
pub(crate) struct IgnoreFile {
    dates: Option<Vec<Datetime>>,
    files: Option<Vec<PathBuf>>,
}

impl IgnoreFile {
    /// Create a new IgnoreFile, regardless of whether one was parsed properly.
    /// Will return an empty IgnoreFile if nothing is found or there was an
    /// error in parsing.
    pub(crate) fn new(path: &Path) -> Self {
        match parse_ignorefile(path) {
            Ok(ignore) => ignore,
            Err(_) => IgnoreFile {
                dates: None,
                files: None,
            },
        }
    }

    pub fn dates(&self) -> &Option<Vec<Datetime>> {
        &self.dates
    }

    pub fn files(&self) -> &Option<Vec<PathBuf>> {
        &self.files
    }
}

pub(crate) fn ignorefile_path_from_dir(dir: &Path) -> PathBuf {
    dir.join(IGNOREFILE)
}

/// Validate the ignore file.
fn validate_ignorefile(path: &Path) -> io::Result<()> {
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Ignore file `{}` not found.", path.display()),
        ));
    }

    if !path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Ignore file `{}` must be a file.", path.display()),
        ));
    }

    Ok(())
}

/// Parse an ignore file and extract the dates and file names.
fn parse_ignorefile(path: &Path) -> io::Result<IgnoreFile> {
    validate_ignorefile(path)?;

    let ignore_str = parse_toml_file(path)?;
    let ignore: IgnoreFile = toml::from_str(&ignore_str)?;

    Ok(ignore)
}

/// Parse a TOML file into a map of values.
fn parse_toml_file(path: &Path) -> io::Result<String> {
    // open the file for parsing
    let mut file = File::open(&path)?;

    // read file contents into a string
    let mut toml_str = String::new();
    file.read_to_string(&mut toml_str)?;

    Ok(toml_str)
}
