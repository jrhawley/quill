//! A collection of all statements for a given account.

use std::collections::HashMap;

use super::ObservedStatement;

/// A survey of all account statements that exist and are required
#[derive(Debug)]
pub struct StatementCollection {
    inner: HashMap<String, Vec<ObservedStatement>>,
}

impl StatementCollection {
    /// Create a new collection of statements.
    pub fn new() -> Self {
        StatementCollection::default()
    }

    /// Access statements belonging to an account
    pub fn get(&self, key: &str) -> Option<&Vec<ObservedStatement>> {
        self.inner.get(key)
    }

    /// Insert statements into the collection
    pub fn insert(&mut self, k: &str, v: Vec<ObservedStatement>) -> Option<Vec<ObservedStatement>> {
        self.inner.insert(k.to_string(), v)
    }
}

impl Default for StatementCollection {
    fn default() -> Self {
        StatementCollection {
            inner: HashMap::new(),
        }
    }
}
