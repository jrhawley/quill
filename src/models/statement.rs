use crate::models::date::Date;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
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
    pub fn from_path(path: &Path, fmt: &str) -> Statement {
        // default to be used with parsing errors
        let false_date = Date::from_ymd(1900, 01, 01);
        let date = Date::parse_from_str(path.file_stem().unwrap().to_str().unwrap(), fmt)
            .unwrap_or(false_date);

        Statement {
            path: PathBuf::from(path),
            date,
        }
    }
}
