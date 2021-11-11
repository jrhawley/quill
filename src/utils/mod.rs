//! Various convenience and utility functions used throughout the codebase.

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

/// Parse a TOML file into a map of values.
pub fn parse_toml_file(path: &Path) -> io::Result<String> {
    // open the file for parsing
    let mut file = File::open(&path)?;

    // read file contents into a string
    let mut toml_str = String::new();
    file.read_to_string(&mut toml_str)?;

    Ok(toml_str)
}
