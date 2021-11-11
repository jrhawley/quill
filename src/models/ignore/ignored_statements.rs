use kronos::Shim;
use serde::Deserialize;
use std::path::Path;

use crate::models::{
    account::{expected_statement_dates, pair_dates_statements},
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
    pub fn new<'a>(first: &Date, period: &Shim<'a>, fmt: &str, dir: &Path) -> Self {
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
        let required_dates = expected_statement_dates(first, period);
        let ignored_date_pairing = match pair_dates_statements(&required_dates, &stmts_from_dates) {
            Ok(v) => v,
            Err(_) => {
                // if any errors are encountered, just return no ignored dates
                vec![]
            }
        };
        let ignored_file_pairing = match pair_dates_statements(&required_dates, &stmts_from_files) {
            Ok(v) => v,
            Err(_) => {
                // if any errors are encountered, just return no ignored dates
                vec![]
            }
        };

        // match the statements from the files with the required statements
        let mut paired_ignore: Vec<Statement> = vec![];
        for (i, _) in required_dates.iter().enumerate() {
            // required_dates, ignored_date_pairing, and ignored_file_pairing
            // are all in the same order, so we can just deal with indices
            match (
                ignored_date_pairing[i].1.as_ref(),
                ignored_file_pairing[i].1.as_ref(),
            ) {
                // ignore the statement as listed by the date if both are specified
                (Some(date_stmt), Some(_)) => paired_ignore.push(date_stmt.clone()),
                // ignore the statement as listed by the date
                (Some(date_stmt), None) => paired_ignore.push(date_stmt.clone()),
                // ignore the statement as listed by the file
                (None, Some(file_stmt)) => paired_ignore.push(file_stmt.clone()),
                (_, _) => {}
            }
        }

        Self {
            stmts: paired_ignore,
        }
    }
}
