//! Utilities to load, parse, and manage the configuration.

use crate::cfg::Config;
use clap::crate_name;
use dirs_next::{config_dir, home_dir};
use quill_statement::StatementCollection;
use std::path::PathBuf;

fn get_config_dir() -> PathBuf {
    // check if $XDG_CONFIG_HOME is set
    match config_dir() {
        Some(dir) => dir,
        // if not set, make it the default $HOME/.config
        None => {
            if let Some(mut dir) = home_dir() {
                dir.push(".config");
                dir
            } else {
                PathBuf::new()
            }
        }
    }
}

/// Check multiple locations for a configuration file and return the highest priority one
pub fn get_config_path() -> PathBuf {
    let mut cfg_path = get_config_dir();

    // get config from within $XDG_CONFIG_HOME
    cfg_path.push(crate_name!().to_lowercase());
    cfg_path.push("config.toml");
    match cfg_path.exists() {
        true => cfg_path,
        false => PathBuf::from("config.toml"),
    }
}

impl<'a> TryFrom<&Config<'a>> for StatementCollection {
    type Error = anyhow::Error;

    fn try_from(value: &Config) -> Result<Self, Self::Error> {
        let mut sc = Self::new();

        for (key, acct) in value.accounts() {
            // generate the vec of required statement dates and statement files
            // (if the statement is available for a given date)
            let matched_stmts = acct.match_statements();
            sc.insert(key, matched_stmts);
        }

        Ok(sc)
    }
}

// Need to reimplement this trait for `&mut Config<'a>` since &T and `&mut T` are different types.
// See https://libreddit.net/r/rust/comments/2a721y/a_safe_way_to_reuse_the_same_code_for_immutable/ for details.
impl<'a> TryFrom<&mut Config<'a>> for StatementCollection {
    type Error = anyhow::Error;

    fn try_from(value: &mut Config) -> Result<Self, Self::Error> {
        let mut sc = Self::new();

        for (key, acct) in value.accounts() {
            // generate the vec of required statement dates and statement files
            // (if the statement is available for a given date)
            let matched_stmts = acct.match_statements();
            sc.insert(key, matched_stmts);
        }

        Ok(sc)
    }
}
