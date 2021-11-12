use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use toml::value::Datetime;

use crate::models::Date;

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Statement {
    path: PathBuf,
    date: Date,
}

impl Statement {
    /// Construct a new Statement
    pub fn new(path: &Path, date: &Date) -> Statement {
        Statement {
            path: path.to_path_buf(),
            date: date.clone(),
        }
    }

    /// Access the date
    pub fn date(&self) -> &Date {
        &self.date
    }

    /// Access the statement path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Construct Statement from a file
    pub fn from_path(path: &Path, fmt: &str) -> Result<Statement, chrono::ParseError> {
        // default to be used with parsing errors
        match Date::parse_from_str(path.file_stem().unwrap().to_str().unwrap(), fmt) {
            Ok(date) => Ok(Statement::new(path, &date)),
            Err(e) => Err(e),
        }
    }

    /// Construct Statement from a date
    pub fn from_date(date: &Date, fmt: &str) -> Result<Statement, chrono::ParseError> {
        let date_str = date.format(fmt).to_string();
        let path = PathBuf::from(date_str);

        Ok(Statement::new(&path, date))
    }

    /// Construct Statement from a date
    pub fn from_datetime(date: &Datetime, fmt: &str) -> Result<Statement, chrono::ParseError> {
        let date = Date::try_from(date)?;
        Statement::from_date(&date, fmt)
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({:?})", self.date(), self.path())
    }
}
