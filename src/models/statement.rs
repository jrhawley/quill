use chrono::ParseError;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::path::{Path, PathBuf};

use crate::models::Date;

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Statement {
    path: PathBuf,
    date: Date,
}

impl Statement {
    /// Construct a new Statement
    pub fn new(path: PathBuf, date: Date) -> Statement {
        Statement { path, date }
    }

    /// Access the date
    pub fn date(&self) -> Date {
        self.date
    }

    /// Access the statement path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Construct Statement from a file
    pub fn from_path(path: &Path, fmt: &str) -> Result<Statement, ParseError> {
        // default to be used with parsing errors
        match Date::parse_from_str(path.file_stem().unwrap().to_str().unwrap(), fmt) {
            Ok(date) => Ok(Statement {
                path: PathBuf::from(path),
                date,
            }),
            Err(e) => Err(e),
        }
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({:?})", self.date(), self.path())
    }
}
