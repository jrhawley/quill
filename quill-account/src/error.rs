//! Error types for this library.

use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AccountCreationError {
    #[error("Missing account name")]
    MissingAccountName,
    #[error("Missing institution name")]
    MissingInstitutionName,
    #[error("Missing statement file name format")]
    MissingStatementFormat,
    #[error("Missing first statement date")]
    MissingFirstDate,
    #[error("Invalid first statement date")]
    InvalidFirstDate(String),
    #[error("Missing statement directory")]
    MissingStatementDirectory,
    #[error("Statement directory `{0}` does not exist")]
    StatementDirectoryNotFound(PathBuf),
    #[error("Error converting statement directory `{0}` to an absolute path")]
    StatementDirectoryNonCanonical(PathBuf),
    #[error("Missing statement period")]
    MissingPeriod,
    #[error("Incorrect array length in statement period (should be 4, was {0}).\nThe required format is `[n, x, m, y]` where `n` and `m` are integers, `x` and `y` are strings.")]
    InvalidPeriodIncorrectLength(usize),
    #[error("Non-integer for `n`th statement period.\nThe required format is `[n, x, m, y]` where `n` and `m` are integers, `x` and `y` are strings.")]
    InvalidPeriodNonIntN,
    #[error("Non-integer for `m`th statement period.\nThe required format is `[n, x, m, y]` where `n` and `m` are integers, `x` and `y` are strings.")]
    InvalidPeriodNonIntM,
    #[error("Incorrect grain string `{0}` for the statement period.\nAllowable grain strings are `Day`, `Week`, `Month`, `Quarter`, `Half`, `Year`, `Lustrum`, `Decade`, `Century`, and `Millenium`.")]
    InvalidPeriodGrainNotAString(String),
    #[error("Incorrect grain string `{0}` for the statement period.\nAllowable grain strings are `Day`, `Week`, `Month`, `Quarter`, `Half`, `Year`, `Lustrum`, `Decade`, `Century`, and `Millenium`.")]
    InvalidPeriodGrainString(String),
    #[error("Unknown error parsing the statement period.\nThe required format is `[n, x, m, y]` where `n` and `m` are integers, `x` and `y` are strings.")]
    InvalidPeriodUnknown,
    #[error("Unknown account data error. This should never happen, please file an issue.")]
    Unknown,
}
