//! The status of an individual statement.

#[derive(Debug)]
pub enum StatementStatus {
    Available,
    Ignored,
    Missing,
}
