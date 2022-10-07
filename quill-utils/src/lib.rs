//! Various convenience and utility functions used throughout the codebase.

use dirs_next::home_dir;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

/// Parse a TOML file into a map of values.
pub fn parse_toml_file(path: &Path) -> io::Result<String> {
    // open the file for parsing
    let mut file = File::open(&path)?;

    // read file contents into a string
    let mut toml_str = String::new();
    file.read_to_string(&mut toml_str)?;

    Ok(toml_str)
}

/// Replace the `~` character at the start of a path with the home directory.
/// See <https://stackoverflow.com/a/54306906/7416009>
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[track_caller]
    fn check_expand_tilde<P: AsRef<Path>>(input: P, expected: Option<PathBuf>) {
        let observed = expand_tilde(input);

        assert_eq!(expected, observed);
    }

    #[test]
    fn test_expand_home_only() {
        let input = Path::new("~");
        let expected = home_dir();

        check_expand_tilde(input, expected);
    }

    #[test]
    fn test_expand_root() {
        let input = Path::new("/");
        let expected = Some(PathBuf::from("/"));

        check_expand_tilde(input, expected);
    }

    #[test]
    fn test_expand_home_plus_child_dir() {
        let input = Path::new("~/Documents");
        let expected = Some(home_dir().unwrap().join("Documents"));

        check_expand_tilde(input, expected);
    }

    #[test]
    fn test_expand_home_plus_sibling_dir() {
        let input = Path::new("~/../Documents");
        let expected = Some(home_dir().unwrap().join("..").join("Documents"));

        check_expand_tilde(input, expected);
    }
}
