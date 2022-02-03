//! Errors and error-handling for the statements.

use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IgnoreFileError {
    #[error("Ignorefile `{0}` not found.")]
    NotFound(PathBuf),
    #[error("Ignorefile must be a file, but `{0}` is not.")]
    NotAFile(PathBuf),
    #[error("Ignorefile `{0}` could not be parsed. Ensure that it is properly formatted.")]
    InvalidIgnorefile(PathBuf),
}
