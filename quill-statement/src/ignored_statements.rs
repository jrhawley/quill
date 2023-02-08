//! A collection of ignored statements.

use crate::ignore_file::{ignorefile_path_from_dir, IgnoreFile};
use chrono::NaiveDate;
use serde::Deserialize;
use std::path::Path;
use std::slice::Iter;
use std::str::FromStr;

/// Control which account statements are ignored.
/// Essentially a sorted `Vec<NaiveDate>`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct IgnoredStatements {
    dates: Vec<NaiveDate>,
}

impl IgnoredStatements {
    /// Construct an empty `IgnoredStatements` object.
    pub fn empty() -> Self {
        Self { dates: vec![] }
    }

    /// Return an iterator over the statements.
    pub fn iter(&self) -> Iter<NaiveDate> {
        self.dates.iter()
    }

    /// Add a date to be ignored.
    pub fn push(&mut self, date: &NaiveDate) {
        let owned_date = date.clone();
        self.dates.push(owned_date);
    }
}

impl From<Vec<NaiveDate>> for IgnoredStatements {
    fn from(v: Vec<NaiveDate>) -> Self {
        Self { dates: v }
    }
}

impl From<&IgnoreFile> for IgnoredStatements {
    fn from(ignore: &IgnoreFile) -> Self {
        match ignore.dates() {
            Some(v) => {
                let mut dates: Vec<NaiveDate> = v
                    .iter()
                    .filter_map(|d| NaiveDate::from_str(&d.to_string()).ok())
                    .collect();

                // ensure the list is sorted so iteration over the Vec is the same as moving forward in time
                dates.sort();

                Self::from(dates)
            }
            None => Self::empty(),
        }
    }
}

impl From<&Path> for IgnoredStatements {
    fn from(path: &Path) -> Self {
        // if the path doesn't exist, just return an empty ignore
        if !path.exists() {
            return Self::from(vec![]);
        }

        // if it's a directory, automatically extract the ignorefile from within
        let ig_file = match path.is_dir() {
            true => ignorefile_path_from_dir(path),
            false => path.to_path_buf(),
        };

        let ig = IgnoreFile::force_new(&ig_file);

        Self::from(&ig)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use toml::value::Datetime;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    fn check_from_vec_naivedate(input: Vec<NaiveDate>, expected: IgnoredStatements) {
        let observed = IgnoredStatements::from(input);

        assert_eq!(expected, observed);
    }

    #[test]
    fn from_empty_vec() {
        let input = vec![];
        let expected = IgnoredStatements::empty();

        check_from_vec_naivedate(input, expected);
    }

    #[test]
    fn from_single_vec() {
        let single_stmt = vec![NaiveDate::from_ymd_opt(2021, 11, 1).unwrap()];
        let input = single_stmt.clone();
        let expected = IgnoredStatements {
            dates: single_stmt.clone(),
        };

        check_from_vec_naivedate(input, expected);
    }

    #[test]
    fn from_double_vec() {
        let double_stmt = vec![
            NaiveDate::from_ymd_opt(2021, 11, 1).unwrap(),
            NaiveDate::from_ymd_opt(2022, 12, 1).unwrap(),
        ];
        let input = double_stmt.clone();
        let expected = IgnoredStatements {
            dates: double_stmt.clone(),
        };

        check_from_vec_naivedate(input, expected);
    }

    fn check_new(input: &IgnoreFile, expected: IgnoredStatements) {
        let observed = IgnoredStatements::from(input);

        assert_eq!(expected, observed);
    }

    #[test]
    fn new_empty() {
        let ignore = IgnoreFile::empty();

        let expected = IgnoredStatements::empty();

        check_new(&ignore, expected);
    }

    #[test]
    fn new_single() {
        let ignore = IgnoreFile::from(vec![Datetime::from_str("2015-07-22").unwrap()]);

        let expected = IgnoredStatements::from(vec![NaiveDate::from_ymd_opt(2015, 7, 22).unwrap()]);

        check_new(&ignore, expected);
    }

    #[test]
    fn new_double_ordered() {
        let ignore = IgnoreFile::from(vec![
            Datetime::from_str("2015-07-22").unwrap(),
            Datetime::from_str("2015-08-22").unwrap(),
        ]);

        let expected = IgnoredStatements::from(vec![
            NaiveDate::from_ymd_opt(2015, 7, 22).unwrap(),
            NaiveDate::from_ymd_opt(2015, 8, 22).unwrap(),
        ]);

        check_new(&ignore, expected);
    }

    #[test]
    fn new_double_unordered() {
        let ignore = IgnoreFile::from(vec![
            Datetime::from_str("2015-08-22").unwrap(),
            Datetime::from_str("2015-07-22").unwrap(),
        ]);

        let expected = IgnoredStatements::from(vec![
            NaiveDate::from_ymd_opt(2015, 7, 22).unwrap(),
            NaiveDate::from_ymd_opt(2015, 8, 22).unwrap(),
        ]);

        check_new(&ignore, expected);
    }

    #[test]
    fn realistic_missing() {
        let ignore = IgnoreFile::from(vec![
            Datetime::from_str("2021-01-22").unwrap(),
            Datetime::from_str("2021-05-25").unwrap(),
            Datetime::from_str("2021-10-22").unwrap(),
        ]);

        let expected = IgnoredStatements {
            dates: vec![
                NaiveDate::from_ymd_opt(2021, 1, 22).unwrap(),
                NaiveDate::from_ymd_opt(2021, 5, 25).unwrap(),
                NaiveDate::from_ymd_opt(2021, 10, 22).unwrap(),
            ],
        };

        check_new(&ignore, expected);
    }
}
