//! A collection of ignored statements.

use chrono::NaiveDate;
use kronos::Shim;
use serde::Deserialize;
use std::{path::Path, slice::Iter};

use crate::ignore_file::{ignorefile_path_from_dir, IgnoreFile};
use crate::{expected_statement_dates, pair_dates_statements, Statement, StatementStatus};

/// Control which account statements are ignored.
#[derive(Clone, Debug, Deserialize)]
pub struct IgnoredStatements {
    // statement dates that are being skipped/ignored
    stmts: Vec<Statement>,
}

impl IgnoredStatements {
    /// Construct an empty `IgnoredStatements` object.
    pub fn empty() -> Self {
        Self { stmts: vec![] }
    }

    /// Construct a new `IgnoredStatements` object.
    pub fn new<'a>(first: &NaiveDate, period: &Shim<'a>, fmt: &str, dir: &Path) -> Self {
        let ignore_path = ignorefile_path_from_dir(dir);
        let ignore_file = IgnoreFile::force_new(ignore_path.as_path());

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

        // match the statements from the dates with the required statements
        let empty_ignore = Self::empty();
        let required_dates = expected_statement_dates(first, period);
        let ignored_date_pairing =
            pair_dates_statements(&required_dates, &stmts_from_dates, &empty_ignore);
        let ignored_file_pairing =
            pair_dates_statements(&required_dates, &stmts_from_files, &empty_ignore);

        // match the statements from the files with the required statements
        let mut paired_ignore: Vec<Statement> = vec![];
        for (i, d) in required_dates.iter().enumerate() {
            // required_dates, ignored_date_pairing, and ignored_file_pairing
            // are all in the same order, so we can just deal with indices
            match (
                ignored_date_pairing[i].status(),
                ignored_file_pairing[i].status(),
            ) {
                // ignore the statement as listed by the date if both are specified
                (StatementStatus::Available, _) => {
                    paired_ignore.push(ignored_date_pairing[i].statement().clone())
                }
                // ignore the statement as listed by the file
                (StatementStatus::Missing, StatementStatus::Available) => {
                    // take the precise date and combine it with the statement file that is ignored
                    // this will make matching the statement easier
                    let new_stmt = Statement::new(ignored_file_pairing[i].statement().path(), d);
                    paired_ignore.push(new_stmt);
                }
                (_, _) => {}
            }
        }

        Self {
            stmts: paired_ignore,
        }
    }

    /// Return an iterator over the statements
    pub fn iter(&self) -> Iter<Statement> {
        self.stmts.iter()
    }
}

impl From<Vec<Statement>> for IgnoredStatements {
    fn from(v: Vec<Statement>) -> Self {
        Self { stmts: v }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    // #[test]
}
