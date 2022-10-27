//! Command line interface configuration.

use clap::Parser;
use lazy_static::lazy_static;
use std::path::{Path, PathBuf};
use crate::cfg::utils::get_config_path;

lazy_static! {
    static ref DEFAULT_CFG_PATH: PathBuf = get_config_path();
}

#[derive(Debug, Parser)]
#[clap(author, about, version)]
pub(crate) struct CliOpts {
    #[clap(
        name = "cfg",
        short,
        long,
        help = "Configuration file with accounts and statements info.",
        default_value = (*DEFAULT_CFG_PATH).as_os_str()
    )]
    config: PathBuf,
}

impl CliOpts {
    /// Retrieve the config file path
    pub fn config(&self) -> &Path {
        &self.config
    }
}
