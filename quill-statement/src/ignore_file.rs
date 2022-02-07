//! Read and parse the ignore files written by the user.

use quill_utils::parse_toml_file;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use toml::value::Datetime;

use crate::IgnoreFileError;

use super::IGNOREFILE;

/// An intermediate format for parsing ignore files.
/// This intermediate exists to simplify deserialization with TOML.
#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct IgnoreFile {
    dates: Option<Vec<Datetime>>,
    files: Option<Vec<PathBuf>>,
}

impl IgnoreFile {
    /// Create a new IgnoreFile, regardless of whether one was parsed properly.
    /// Will return an empty IgnoreFile if nothing is found or there was an
    /// error in parsing.
    pub(crate) fn force_new(path: &Path) -> Self {
        match IgnoreFile::try_from(path) {
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

pub(crate) fn ignorefile_path_from_dir(dir: &Path) -> PathBuf {
    dir.join(IGNOREFILE)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    fn check_try_from_path(input_path: &Path, expected: Result<IgnoreFile, IgnoreFileError>) {
        let observed = IgnoreFile::try_from(input_path);
        assert_eq!(expected, observed);
    }

    #[test]
    fn no_dates_no_files() {
        let ignorefile = Path::new("tests/no_dates_no_files.toml");
        let expected = IgnoreFile {
            dates: None,
            files: None,
        };

        check_try_from_path(ignorefile, Ok(expected));
    }

    #[test]
    fn empty_dates_no_files() {
        let ignorefile = Path::new("tests/empty_dates_no_files.toml");
        let expected = IgnoreFile {
            dates: Some(vec![]),
            files: None,
        };

        check_try_from_path(ignorefile, Ok(expected));
    }

    #[test]
    fn no_dates_empty_files() {
        let ignorefile = Path::new("tests/no_dates_empty_files.toml");
        let expected = IgnoreFile {
            dates: None,
            files: Some(vec![]),
        };

        check_try_from_path(ignorefile, Ok(expected));
    }

    #[test]
    fn empty_dates_empty_files() {
        let ignorefile = Path::new("tests/empty_dates_empty_files.toml");
        let expected = IgnoreFile {
            dates: Some(vec![]),
            files: Some(vec![]),
        };

        check_try_from_path(ignorefile, Ok(expected));
    }

    #[test]
    fn some_dates_no_files() {
        let ignorefile = Path::new("tests/some_dates_no_files.toml");
        let expected = IgnoreFile {
            dates: Some(vec![Datetime::from_str("2021-11-01").unwrap()]),
            files: None,
        };

        check_try_from_path(ignorefile, Ok(expected));
    }

    #[test]
    #[should_panic]
    fn error_dates_no_files() {
        let ignorefile = Path::new("tests/error_dates_no_files.toml");
        let expected = IgnoreFile {
            dates: Some(vec![Datetime::from_str("2021-11-01").unwrap()]),
            files: None,
        };

        check_try_from_path(ignorefile, Ok(expected));
    }

    #[test]
    fn no_dates_some_files() {
        let ignorefile = Path::new("tests/no_dates_some_files.toml");
        let expected = IgnoreFile {
            dates: None,
            files: Some(vec![PathBuf::from("2021-11-01.pdf")]),
        };

        check_try_from_path(ignorefile, Ok(expected));
    }

    #[test]
    #[should_panic]
    fn no_dates_error_files() {
        let ignorefile = Path::new("tests/no_dates_error_files.toml");
        let expected = IgnoreFile {
            dates: None,
            files: Some(vec![PathBuf::from("2021-11-01.pdf")]),
        };

        check_try_from_path(ignorefile, Ok(expected));
    }

    #[test]
    fn empty_dates_some_files() {
        let ignorefile = Path::new("tests/empty_dates_some_files.toml");
        let expected = IgnoreFile {
            dates: Some(vec![]),
            files: Some(vec![PathBuf::from("2021-11-01.pdf")]),
        };

        check_try_from_path(ignorefile, Ok(expected));
    }

    #[test]
    fn some_dates_empty_files() {
        let ignorefile = Path::new("tests/some_dates_empty_files.toml");
        let expected = IgnoreFile {
            dates: Some(vec![Datetime::from_str("2021-11-01").unwrap()]),
            files: Some(vec![]),
        };

        check_try_from_path(ignorefile, Ok(expected));
    }

    #[test]
    fn some_dates_some_files() {
        let ignorefile = Path::new("tests/some_dates_some_files.toml");
        let expected = IgnoreFile {
            dates: Some(vec![Datetime::from_str("2021-11-01").unwrap()]),
            files: Some(vec![PathBuf::from("2021-11-01.pdf")]),
        };

        check_try_from_path(ignorefile, Ok(expected));
    }

    #[test]
    fn nonoverlapping_dates_files() {
        let ignorefile = Path::new("tests/non-overlapping_dates_files.toml");
        let expected = IgnoreFile {
            dates: Some(vec![Datetime::from_str("2021-11-01").unwrap()]),
            files: Some(vec![PathBuf::from("2021-12-01.pdf")]),
        };

        check_try_from_path(ignorefile, Ok(expected));
    }
}
