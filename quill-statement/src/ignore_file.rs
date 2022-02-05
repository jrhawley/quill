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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    fn check_parse_ignorefile(input_path: &Path, expected: Result<IgnoreFile, IgnoreFileError>) {
        let observed = parse_ignorefile(input_path);
        assert_eq!(expected, observed);
    }

    #[test]
    fn no_dates_no_files() {
        let ignorefile = Path::new("tests/no_dates_no_files.toml");
        let expected = IgnoreFile {
            dates: None,
            files: None,
        };

        check_parse_ignorefile(ignorefile, Ok(expected));
    }

    #[test]
    fn empty_dates_no_files() {
        let ignorefile = Path::new("tests/empty_dates_no_files.toml");
        let expected = IgnoreFile {
            dates: Some(vec![]),
            files: None,
        };

        check_parse_ignorefile(ignorefile, Ok(expected));
    }

    #[test]
    fn no_dates_empty_files() {
        let ignorefile = Path::new("tests/no_dates_empty_files.toml");
        let expected = IgnoreFile {
            dates: None,
            files: Some(vec![]),
        };

        check_parse_ignorefile(ignorefile, Ok(expected));
    }

    #[test]
    fn empty_dates_empty_files() {
        let ignorefile = Path::new("tests/empty_dates_empty_files.toml");
        let expected = IgnoreFile {
            dates: Some(vec![]),
            files: Some(vec![]),
        };

        check_parse_ignorefile(ignorefile, Ok(expected));
    }

    #[test]
    fn some_dates_no_files() {
        let ignorefile = Path::new("tests/some_dates_no_files.toml");
        let expected = IgnoreFile {
            dates: Some(vec![Datetime::from_str("2021-11-01").unwrap()]),
            files: None,
        };

        check_parse_ignorefile(ignorefile, Ok(expected));
    }

    #[test]
    #[should_panic]
    fn error_dates_no_files() {
        let ignorefile = Path::new("tests/error_dates_no_files.toml");
        let expected = IgnoreFile {
            dates: Some(vec![Datetime::from_str("2021-11-01").unwrap()]),
            files: None,
        };

        check_parse_ignorefile(ignorefile, Ok(expected));
    }

    #[test]
    fn no_dates_some_files() {
        let ignorefile = Path::new("tests/no_dates_some_files.toml");
        let expected = IgnoreFile {
            dates: None,
            files: Some(vec![PathBuf::from("2021-11-01.pdf")]),
        };

        check_parse_ignorefile(ignorefile, Ok(expected));
    }

    #[test]
    #[should_panic]
    fn no_dates_error_files() {
        let ignorefile = Path::new("tests/no_dates_error_files.toml");
        let expected = IgnoreFile {
            dates: None,
            files: Some(vec![PathBuf::from("2021-11-01.pdf")]),
        };

        check_parse_ignorefile(ignorefile, Ok(expected));
    }

    #[test]
    fn empty_dates_some_files() {
        let ignorefile = Path::new("tests/empty_dates_some_files.toml");
        let expected = IgnoreFile {
            dates: Some(vec![]),
            files: Some(vec![PathBuf::from("2021-11-01.pdf")]),
        };

        check_parse_ignorefile(ignorefile, Ok(expected));
    }

    #[test]
    fn some_dates_empty_files() {
        let ignorefile = Path::new("tests/some_dates_empty_files.toml");
        let expected = IgnoreFile {
            dates: Some(vec![Datetime::from_str("2021-11-01").unwrap()]),
            files: Some(vec![]),
        };

        check_parse_ignorefile(ignorefile, Ok(expected));
    }

    #[test]
    fn some_dates_some_files() {
        let ignorefile = Path::new("tests/some_dates_some_files.toml");
        let expected = IgnoreFile {
            dates: Some(vec![Datetime::from_str("2021-11-01").unwrap()]),
            files: Some(vec![PathBuf::from("2021-11-01.pdf")]),
        };

        check_parse_ignorefile(ignorefile, Ok(expected));
    }
}
