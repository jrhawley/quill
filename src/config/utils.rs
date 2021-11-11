//! Utilities to load, parse, and manage the configuration.

use clap::crate_name;
use dirs::{config_dir, home_dir};
use std::{env, io::Result, path::PathBuf};
use toml::map::Map;
use toml::Value;

use crate::config::Config;

/// Parse a TOML table for accounts and create Accounts
pub(crate) fn parse_accounts<'a, 'b>(
    accounts: &'a Map<String, Value>,
    conf: &'a mut Config<'b>,
) -> Result<()> {
    for (acct, props) in accounts {
        match conf.add_account(acct, props) {
            Ok(_) => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

/// Check multiple locations for a configuration file and return the highest priority one
pub fn get_config_path() -> PathBuf {
    // check if $XDG_CONFIG_HOME is set
    let mut cfg_path = match config_dir() {
        Some(dir) => PathBuf::from(dir),
        // if not set, make it the default $HOME/.config
        None => {
            if let Some(mut dir) = home_dir() {
                dir.push(".config");
                dir
            } else {
                PathBuf::new()
            }
        }
    };

    // get config from within $XDG_CONFIG_HOME
    cfg_path.push(crate_name!().to_lowercase());
    cfg_path.push("config.toml");
    match cfg_path.exists() {
        true => cfg_path,
        false => PathBuf::from("config.toml"),
    }
}
