//! A helper object to keep track of everything about a statement.
//! This includes what date it's supposed to correspond to, the statement file as given or expected, and its status.

use crate::models::Date;

use super::StatementStatus;

#[derive(Debug)]
pub struct ObservedStatement {
    date: Date,
    status: StatementStatus,
}

impl ObservedStatement {
    pub fn new(date: &Date, status: StatementStatus) -> Self {
        Self {
            date: (*date).clone(),
            status
        }
    }
}
