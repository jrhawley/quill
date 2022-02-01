//! A helper object to keep track of everything about a statement.
//! This includes what date it's supposed to correspond to, the statement file as given or expected, and its status.

use super::{Statement, StatementStatus};

#[derive(Debug, PartialEq)]
pub struct ObservedStatement {
    stmt: Statement,
    status: StatementStatus,
}

impl ObservedStatement {
    pub fn new(stmt: &Statement, status: StatementStatus) -> Self {
        Self {
            stmt: (*stmt).clone(),
            status
        }
    }

    pub fn statement(&self) -> &Statement {
        &self.stmt
    }

    pub fn status(&self) -> StatementStatus {
        self.status
    }
}
