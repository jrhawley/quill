//! Read and parse the ignore files written by the user.

use crate::IgnoreFileError;
use quill_utils::parse_toml_file;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use toml::value::Datetime;

const IGNOREFILE: &str = ".quillignore.toml";

/// An intermediate format for parsing ignore files.
/// This intermediate exists to simplify deserialization with TOML.
/// In practice, it should be immediately transformed into an `IgnoredStatements`.
#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct IgnoreFile {
    dates: Option<Vec<Datetime>>,
}

impl IgnoreFile {
    /// Create a new empty IgnoreFile that doesn't have the dates anywhere
    pub fn missing() -> Self {
        IgnoreFile { dates: None }
    }

    /// Create a new IgnoreFile from an empty array
    pub fn empty() -> Self {
        IgnoreFile {
            dates: Some(vec![]),
        }
    }

    /// Create a new IgnoreFile, regardless of whether one was parsed properly.
    /// Will return an empty IgnoreFile if nothing is found or there was an
    /// error in parsing.
    pub fn force_new(path: &Path) -> Self {
        IgnoreFile::try_from(path).unwrap_or_else(|_| Self::empty())
    }

    pub fn dates(&self) -> &Option<Vec<Datetime>> {
        &self.dates
    }
}

impl From<Vec<Datetime>> for IgnoreFile {
    fn from(v: Vec<Datetime>) -> Self {
        Self { dates: Some(v) }
    }
}

impl TryFrom<&str> for IgnoreFile {
    type Error = IgnoreFileError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match toml::from_str(value) {
            Ok(i) => Ok(i),
            Err(_) => return Err(IgnoreFileError::InvalidIgnorefileString(value.to_string())),
        }
    }
}

impl TryFrom<&Path> for IgnoreFile {
    type Error = IgnoreFileError;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        if !path.exists() {
            return Err(IgnoreFileError::NotFound(path.to_path_buf()));
        }

        if !path.is_file() {
            return Err(IgnoreFileError::NotAFile(path.to_path_buf()));
        }

        let ignore_str = match parse_toml_file(path) {
            Ok(s) => s,
            Err(_) => return Err(IgnoreFileError::InvalidIgnorefile(path.to_path_buf())),
        };

        IgnoreFile::try_from(ignore_str.as_str())
    }
}

pub fn ignorefile_path_from_dir(dir: &Path) -> PathBuf {
    dir.join(IGNOREFILE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn check_missing() {
        let observed = IgnoreFile::missing();
        let expected = IgnoreFile { dates: None };

        assert_eq!(expected, observed);
    }

    fn check_try_from_path(input_path: &Path, expected: Result<IgnoreFile, IgnoreFileError>) {
        let observed = IgnoreFile::try_from(input_path);
        assert_eq!(expected, observed);
    }

    #[test]
    fn no_dates() {
        let ignorefile = Path::new("tests/no_dates.toml");
        let expected = IgnoreFile::missing();

        check_try_from_path(ignorefile, Ok(expected));
    }

    #[test]
    fn empty_dates() {
        let ignorefile = Path::new("tests/empty_dates.toml");
        let expected = IgnoreFile::empty();

        check_try_from_path(ignorefile, Ok(expected));
    }

    #[test]
    fn one_date() {
        let ignorefile = Path::new("tests/one_date.toml");
        let expected = IgnoreFile::from(vec![Datetime::from_str("2021-11-01").unwrap()]);

        check_try_from_path(ignorefile, Ok(expected));
    }

    #[test]
    fn some_dates() {
        let ignorefile = Path::new("tests/some_dates.toml");
        let expected = IgnoreFile::from(vec![
            Datetime::from_str("2021-11-01").unwrap(),
            Datetime::from_str("2021-12-01").unwrap(),
        ]);

        check_try_from_path(ignorefile, Ok(expected));
    }

    #[test]
    #[should_panic]
    fn error_dates() {
        let ignorefile = Path::new("tests/error_dates.toml");
        let expected = IgnoreFile::from(vec![Datetime::from_str("2021-11-01").unwrap()]);

        check_try_from_path(ignorefile, Ok(expected));
    }
}
