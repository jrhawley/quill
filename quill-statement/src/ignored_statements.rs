//! A collection of ignored statements.

use chrono::NaiveDate;
use kronos::Shim;
use serde::Deserialize;
use std::slice::Iter;

use crate::ignore_file::IgnoreFile;
use crate::{expected_statement_dates, pair_dates_statements, Statement, StatementStatus};

/// Control which account statements are ignored.
#[derive(Clone, Debug, Deserialize, PartialEq)]
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
    pub fn new<'a>(first: &NaiveDate, period: &Shim<'a>, fmt: &str, ignore: &IgnoreFile) -> Self {
        let stmts_from_dates: Vec<Statement> = match ignore.dates() {
            Some(d) => d
                .iter()
                .filter_map(|d| Statement::try_from((d, fmt)).ok())
                .collect(),
            None => vec![],
        };

        // match the statements from the dates with the required statements
        let empty_ignore = Self::empty();
        let required_dates = expected_statement_dates(first, period);
        let ignored_date_pairing =
            pair_dates_statements(&required_dates, &stmts_from_dates, &empty_ignore);

        // match the statements from the files with the required statements
        let mut paired_ignore: Vec<Statement> = vec![];
        for (i, _) in required_dates.iter().enumerate() {
            // required_dates, ignored_date_pairing, and ignored_file_pairing
            // are all in the same order, so we can just deal with indices
            match ignored_date_pairing[i].status() {
                // ignore the statement as listed by the date if both are specified
                StatementStatus::Available => {
                    paired_ignore.push(ignored_date_pairing[i].statement().clone())
                }
                _ => {}
            }
        }

        Self::from(paired_ignore)
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
    use std::str::FromStr;

    use super::*;
    use kronos::{Grain, Grains, NthOf};
    use toml::value::Datetime;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    fn check_new(input: (&NaiveDate, &Shim, &str, &IgnoreFile), expected: IgnoredStatements) {
        let first = input.0;
        let period = input.1;
        let fmt = input.2;
        let ignore = input.3;

        let observed = IgnoredStatements::new(first, period, fmt, ignore);

        assert_eq!(expected, observed);
    }

    #[test]
    fn realistic_missing() {
        let first = NaiveDate::from_ymd(2015, 07, 24);
        let period = Shim::new(NthOf(22, Grains(Grain::Day), Grains(Grain::Month)));
        let fmt = "%Y-%m-%d";
        let ignore = IgnoreFile::from(vec![
            Datetime::from_str("2021-01-22").unwrap(),
            Datetime::from_str("2021-05-25").unwrap(),
            Datetime::from_str("2021-10-22").unwrap(),
        ]);

        let expected = IgnoredStatements {
            stmts: vec![
                Statement::try_from((&NaiveDate::from_ymd(2021, 1, 22), fmt)).unwrap(),
                Statement::try_from((&NaiveDate::from_ymd(2021, 5, 25), fmt)).unwrap(),
                Statement::try_from((&NaiveDate::from_ymd(2021, 10, 22), fmt)).unwrap(),
            ],
        };

        check_new((&first, &period, fmt, &ignore), expected);
    }
}
