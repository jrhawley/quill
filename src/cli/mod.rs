//! Command line interface configuration.

use clap::{
    app_from_crate, crate_authors, crate_description, crate_name, crate_version, Arg, ArgMatches,
};
use std::{
    io,
    path::{Path, PathBuf},
};

use crate::config::{config::Config, utils::get_config_path};

/// Parse the CLI
pub(crate) fn cli_extract_cfg<'a>() -> io::Result<Config<'a>> {
    let cfg_path = get_config_path();
    let matches = get_cli_matches(cfg_path.as_path());
    validate_cli(&matches)?;
    match cli_extract_conf_path(&matches) {
        Ok(p) => Config::new_from_path(p.as_path()),
        Err(e) => Err(e),
    }
}

/// Get the matches from the CLI
fn get_cli_matches<'a>(default_cfg: &'a Path) -> ArgMatches<'a> {
    // convert default config file path into a string for ArgMatches
    let display_path = default_cfg.to_str().unwrap_or("");

    // CLI interface for binary
    let matches = app_from_crate!()
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("CONF")
                .help("The statement configuration file")
                .takes_value(true)
                .default_value(display_path),
        )
        .get_matches();

    matches
}

/// Validate the given CLI arguments
fn validate_cli(matches: &ArgMatches) -> io::Result<()> {
    // extract the value of the config file path
    match cli_extract_conf_path(matches) {
        Ok(p) => {}
        Err(e) => return Err(e),
    }
    Ok(())
}

/// Extract the configuration file path from the CLI arguments
fn cli_extract_conf_path(matches: &ArgMatches) -> io::Result<PathBuf> {
    match matches.value_of("config") {
        Some(p) => Ok(PathBuf::from(p)),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No configuration file given.",
            ))
        }
    }
}
