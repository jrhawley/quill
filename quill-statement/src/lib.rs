//! Parse, read, and keep track of account statements.

/// File within the account's directory that lists what statement dates
/// should be ignored.
pub(crate) const IGNOREFILE: &str = ".quillignore.toml";

mod ignore_file;
mod ignored_statements;
mod observed_statement;
mod statement_collection;
mod statement_ops;
mod statement_status;
mod statement_struct;

pub use ignored_statements::IgnoredStatements;
pub use observed_statement::ObservedStatement;
pub use statement_collection::StatementCollection;
pub use statement_ops::{
    next_date_from_given, next_date_from_today, next_weekday_date, prev_date_from_given,
    prev_date_from_today,
};
pub use statement_status::StatementStatus;
pub use statement_struct::Statement;
