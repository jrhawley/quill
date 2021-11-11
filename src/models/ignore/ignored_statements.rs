use chrono::ParseError;
use kronos::Shim;
use serde::Deserialize;
use std::{convert::TryFrom, path::Path};
use toml::value::Datetime;

use crate::models::{
    ignore::ignore_file::{ignorefile_path_from_dir, IgnoreFile},
    Date, Statement,
};

/// Control which account statements are ignored.
#[derive(Clone, Debug, Deserialize)]
pub struct IgnoredStatements {
    // statement dates that are being skipped/ignored
    stmts: Vec<Statement>,
}

impl IgnoredStatements {
    /// Construct a new `IgnoredStatements` object.
    pub fn new<'a>(first: Date, period: &Shim<'a>, fmt: &str, dir: &Path) -> Self {
        let ignore_path = ignorefile_path_from_dir(dir);
        let ignore_file = IgnoreFile::new(ignore_path.as_path());

        let stmts_from_dates: Vec<Statement> = match ignore_file.dates() {
            Some(v) => v
                .iter()
                .filter_map(|d| Statement::from_datetime(d, fmt).ok())
                .collect(),
            None => vec![],
        };

        let stmts_from_files: Vec<Statement> = match ignore_file.files() {
            Some(v) => v
                .iter()
                .filter_map(|f| Statement::from_path(f.as_path(), fmt).ok())
                .collect(),
            None => vec![],
        };

        println!("{:#?}", stmts_from_dates);
        println!("{:#?}", stmts_from_files);

        // let filename_fmt_path = dir.join(fmt);
        // let filename_fmt = filename_fmt_path.to_str().unwrap_or("");

        // let path = match dir.is_dir() {
        //     true => Some(dir.join(IGNOREFILE)),
        //     false => None,
        // };

        // let dates = match path {
        //     Some(ref ignorefile_path) => match parse_ignorefile(ignorefile_path.as_path()) {
        //         Ok(v) => v,
        //         Err(_) => vec![],
        //     },
        //     None => vec![],
        // };

        // let stmts = dates
        //     .iter()
        //     .filter_map(|&d| Statement::from_date(d, filename_fmt).ok())
        //     .collect();

        // Self { stmts }
        unimplemented!()
    }
}
