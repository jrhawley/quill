use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::Display;
use std::io;
use std::path::{Path, PathBuf};
use toml::value::Datetime;

use crate::config::Config;
use crate::models::Date;

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Statement {
    path: PathBuf,
    date: Date,
}

impl Statement {
    /// Construct a new Statement
    pub fn new(path: &Path, date: Date) -> Statement {
        Statement {
            path: path.to_path_buf(),
            date,
        }
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
    pub fn from_path(path: &Path, fmt: &str) -> Result<Statement, chrono::ParseError> {
        // default to be used with parsing errors
        match Date::parse_from_str(path.file_stem().unwrap().to_str().unwrap(), fmt) {
            Ok(date) => Ok(Statement::new(path, date)),
            Err(e) => Err(e),
        }
    }

    /// Construct Statement from a date
    pub fn from_date(date: Date, fmt: &str) -> Result<Statement, chrono::ParseError> {
        let date_str = date.format(fmt).to_string();
        let path = PathBuf::from(date_str);

        Ok(Statement::new(&path, date))
    }

    /// Construct Statement from a date
    pub fn from_datetime(date: &Datetime, fmt: &str) -> Result<Statement, chrono::ParseError> {
        let date = Date::try_from(date)?;
        Statement::from_date(date, fmt)
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({:?})", self.date(), self.path())
    }
}

/// A survey of all account statements that exist and are required
#[derive(Debug)]
pub struct StatementCollection {
    inner: HashMap<String, Vec<(Date, Option<Statement>)>>,
}

impl StatementCollection {
    /// Create a new collection of statements.
    pub fn new() -> Self {
        StatementCollection::default()
    }

    /// Derive a new collection of statements from a configuration.
    pub fn new_from_config(conf: &Config) -> io::Result<Self> {
        let mut sc = Self::new();

        for (key, acct) in conf.accounts() {
            // generate the vec of required statement dates and statement files
            // (if the statement is available for a given date)
            let matched_stmts = acct.match_statements()?;
            sc.inner.insert(key.to_string(), matched_stmts);
        }

        Ok(sc)
    }

    /// Access statements belonging to an account
    pub fn get(&self, key: &str) -> Option<&Vec<(Date, Option<Statement>)>> {
        self.inner.get(key)
    }
}

impl Default for StatementCollection {
    fn default() -> Self {
        StatementCollection {
            inner: HashMap::new(),
        }
    }
}
