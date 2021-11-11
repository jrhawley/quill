//! Find and handle which statements should be ignored.

mod ignore_file;
mod ignored_statements;

pub use ignored_statements::IgnoredStatements;

/// File within the account's directory that lists what statement dates
/// should be ignored.
pub(crate) const IGNOREFILE: &str = ".quillignore.toml";
