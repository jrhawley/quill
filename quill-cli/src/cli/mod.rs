//! Command line interface configuration.

use clap::Parser;
use lazy_static::lazy_static;
use std::path::{Path, PathBuf};

use crate::cfg::utils::get_config_path;

/// Get the default configuration path used for the CLI
pub fn default_config_path() -> String {
    let cfg_path = get_config_path();
    let cfg_path_str = cfg_path.to_str().unwrap().to_owned();

    cfg_path_str
}

lazy_static! {
    static ref DEFAULT_CFG_PATH: String = default_config_path();
}

#[derive(Debug, Parser)]
#[clap(author, about, version)]
pub(crate) struct CliOpts {
    #[clap(name = "cfg", short, long, help = "Configuration file with accounts and statements info.", default_value = &DEFAULT_CFG_PATH)]
    config: PathBuf,
}

impl CliOpts {
    /// Retrieve the config file path
    pub fn config(&self) -> &Path {
        &self.config
    }
}
