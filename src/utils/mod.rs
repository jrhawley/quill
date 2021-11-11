//! Various convenience and utility functions used throughout the codebase.

use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use dirs::home_dir;

/// Replace the `~` character in any path with the home directory
/// See https://stackoverflow.com/a/54306906/7416009
pub fn expand_tilde<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
    let p = path.as_ref();
    if !p.starts_with("~") {
        return Some(p.to_path_buf());
    }
    if p == Path::new("~") {
        return home_dir();
    }
    home_dir().map(|mut h| {
        if h == Path::new("/") {
            // base case: `h` root directory;
            // don't prepend extra `/`, just drop the tilde.
            p.strip_prefix("~").unwrap().to_path_buf()
        } else {
            h.push(p.strip_prefix("~/").unwrap());
            h
        }
    })
}

/// Parse a TOML file into a map of values.
pub fn parse_toml_file(path: &Path) -> io::Result<String> {
    // open the file for parsing
    let mut file = File::open(&path)?;

    // read file contents into a string
    let mut toml_str = String::new();
    file.read_to_string(&mut toml_str)?;

    Ok(toml_str)
}
