//! The status of an individual statement.

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum StatementStatus {
    Available,
    Ignored,
    Missing,
}

impl From<StatementStatus> for String {
    fn from(status: StatementStatus) -> String {
        match status {
            StatementStatus::Available => String::from("✔"),
            StatementStatus::Ignored => String::from("-"),
            StatementStatus::Missing => String::from("❌"),
        }
    }
}
