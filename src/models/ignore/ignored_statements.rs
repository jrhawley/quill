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

        // match the statements from the dates with the required statements

        // match the statements from the files with the required statements

        // identify conflicts between the two sources using a HashMap<Date, (Statement, Statement)>
        // this will find duplicates between them, then only return a single set of valid statements to be ignored

        // Self { stmts }
        unimplemented!()
    }
}
