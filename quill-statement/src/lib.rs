//! Parse, read, and keep track of account statements.

mod error;
mod ignore_file;
mod ignored_statements;
mod observed_statement;
mod ops;
mod statement_collection;
mod statement_status;
mod statement_struct;

pub use error::{IgnoreFileError, PairingError};
pub use ignored_statements::IgnoredStatements;
pub use observed_statement::ObservedStatement;
pub use ops::{
    expected_statement_dates, next_date_from_given, next_date_from_today, next_weekday_date,
    pair_dates_statements, prev_date_from_given, prev_date_from_today,
};
pub use statement_collection::StatementCollection;
pub use statement_status::StatementStatus;
pub use statement_struct::Statement;
