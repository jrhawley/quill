//! A collection of all statements for a given account.

use std::{collections::HashMap, io};

use crate::{config::Config, models::Date};

use super::Statement;

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
