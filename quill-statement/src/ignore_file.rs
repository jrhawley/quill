//! Read and parse the ignore files written by the user.

use quill_utils::parse_toml_file;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use toml::value::Datetime;

use crate::IgnoreFileError;

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
fn validate_ignorefile(path: &Path) -> Result<(), IgnoreFileError> {
    if !path.exists() {
        return Err(IgnoreFileError::NotFound(path.to_path_buf()));
    }

    if !path.is_file() {
        return Err(IgnoreFileError::NotAFile(path.to_path_buf()));
    }

    Ok(())
}

/// Parse an ignore file and extract the dates and file names.
fn parse_ignorefile(path: &Path) -> Result<IgnoreFile, IgnoreFileError> {
    validate_ignorefile(path)?;

    let ignore_str = match parse_toml_file(path) {
        Ok(s) => s,
        Err(_) => return Err(IgnoreFileError::InvalidIgnorefile(path.to_path_buf())),
    };
    let ignore: IgnoreFile = match toml::from_str(&ignore_str) {
        Ok(i) => i,
        Err(_) => return Err(IgnoreFileError::InvalidIgnorefile(path.to_path_buf())),
    };

    Ok(ignore)
}
