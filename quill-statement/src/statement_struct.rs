use chrono::{self, NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use toml::value::Datetime;

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Statement {
    path: PathBuf,
    date: NaiveDate,
}

impl Statement {
    /// Construct a new Statement
    pub fn new(path: &Path, date: &NaiveDate) -> Statement {
        Statement {
            path: path.to_path_buf(),
            date: date.clone(),
        }
    }

    /// Access the date
    pub fn date(&self) -> &NaiveDate {
        &self.date
    }

    /// Access the statement path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Construct Statement from a file
    pub fn from_path(path: &Path, fmt: &str) -> Result<Statement, chrono::ParseError> {
        // default to be used with parsing errors
        match NaiveDate::parse_from_str(path.file_stem().unwrap().to_str().unwrap(), fmt) {
            Ok(date) => Ok(Statement::new(path, &date)),
            Err(e) => Err(e),
        }
    }

    /// Construct Statement from a date
    pub fn from_date(date: &NaiveDate, fmt: &str) -> Result<Statement, chrono::ParseError> {
        let date_str = date.format(fmt).to_string();
        let path = PathBuf::from(date_str);

        Ok(Statement::new(&path, date))
    }

    /// Construct Statement from a datetime
    pub fn from_datetime(date: &Datetime, fmt: &str) -> Result<Statement, chrono::ParseError> {
        // toml::Datetime currently (as of 2022-02-01) only supports the `.to_string()` accessor.
        // there is some debate about updating this, but this will work for now instead of
        // redefining an entire Date/Datetime type
        let datetime = NaiveDateTime::from_str(&date.to_string())?;
        let parsed_date = datetime.date();

        Statement::from_date(&parsed_date, fmt)
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({:?})", self.date(), self.path())
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use std::{
        path::{Path, PathBuf},
        str::FromStr,
    };
    use toml::value::Datetime;

    use crate::Statement;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    fn check_from_path(
        input_path: &Path,
        input_fmt: &str,
        expected: Result<Statement, chrono::ParseError>,
    ) {
        let observed = Statement::from_path(input_path, input_fmt);
        assert_eq!(expected, observed);
    }

    #[test]
    fn from_path_matching_format() {
        let input_path = PathBuf::from("2021-11-01.pdf");
        let input_fmt = "%Y-%m-%d";
        let expected_date = NaiveDate::from_ymd(2021, 11, 1);
        let expected = Statement::new(&input_path, &expected_date);

        check_from_path(&input_path, input_fmt, Ok(expected));
    }

    #[test]
    #[should_panic]
    fn from_path_mismatching_format() {
        let input_path = PathBuf::from("2021-11-01.pdf");
        let input_fmt = "not-the-right-format-%Y-%m-%d";
        let expected_date = NaiveDate::from_ymd(2021, 11, 1);
        let expected = Statement::new(&input_path, &expected_date);

        check_from_path(&input_path, input_fmt, Ok(expected));
    }

    fn check_from_date(
        input_date: &NaiveDate,
        input_fmt: &str,
        expected: Result<Statement, chrono::ParseError>,
    ) {
        let observed = Statement::from_date(input_date, input_fmt);
        assert_eq!(expected, observed);
    }

    #[test]
    fn from_date_matching_format() {
        let input_date = NaiveDate::from_ymd(2021, 11, 1);
        let input_fmt = "%Y-%m-%d.pdf";

        let expected_path = PathBuf::from("2021-11-01.pdf");
        let expected = Statement::new(&expected_path, &input_date);

        check_from_date(&input_date, input_fmt, Ok(expected));
    }

    #[test]
    #[should_panic]
    fn from_date_mismatching_format() {
        let input_date = NaiveDate::from_ymd(2021, 11, 1);
        let input_fmt = "%Y-.pdf";
        let expected_path = PathBuf::from("2021-11-01.pdf");
        let expected = Statement::new(&expected_path, &input_date);

        check_from_date(&input_date, input_fmt, Ok(expected));
    }

    fn check_from_datetime(
        input_datetime: &Datetime,
        input_fmt: &str,
        expected: Result<Statement, chrono::ParseError>,
    ) {
        let observed = Statement::from_datetime(input_datetime, input_fmt);
        assert_eq!(expected, observed);
    }

    #[test]
    fn from_datetime_matching_format() {
        let input_datetime = Datetime::from_str("2021-11-01 00:00:00").unwrap();
        let input_fmt = "%Y-%m-%d.pdf";

        let expected_date = NaiveDate::from_ymd(2021, 11, 1);
        let expected_path = PathBuf::from("2021-11-01.pdf");
        let expected = Statement::new(&expected_path, &expected_date);

        check_from_datetime(&input_datetime, input_fmt, Ok(expected));
    }

    #[test]
    #[should_panic]
    fn from_datetime_mismatching_format() {
        let input_datetime = Datetime::from_str("2021-11-01 00:00:00").unwrap();
        let input_fmt = "%Y-.pdf";

        let expected_date = NaiveDate::from_ymd(2021, 11, 1);
        let expected_path = PathBuf::from("2021-11-01.pdf");
        let expected = Statement::new(&expected_path, &expected_date);

        check_from_datetime(&input_datetime, input_fmt, Ok(expected));
    }
}
