//! Financial statements.

use chrono::{self, NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use toml::value::Datetime;

pub(crate) const STATEMENT_DEFAULT_PATH_FMT: &str = "";

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
}

impl From<&NaiveDate> for Statement {
    /// Create a new statement just from a date, without a file path string format.
    fn from(date: &NaiveDate) -> Self {
        let temp_path_str = date.format(STATEMENT_DEFAULT_PATH_FMT).to_string();
        let temp_path = Path::new(&temp_path_str);

        Statement::new(temp_path, &date)
    }
}

impl TryFrom<(&Datetime, &str)> for Statement {
    type Error = chrono::ParseError;

    fn try_from(value: (&Datetime, &str)) -> Result<Self, Self::Error> {
        let date = value.0;
        let fmt = value.1;

        // toml::Datetime currently (as of 2022-02-01) only supports the `.to_string()` accessor.
        // there is some debate about updating this, but this will work for now instead of
        // redefining an entire Date/Datetime type
        let datetime = NaiveDateTime::from_str(&date.to_string())?;
        let parsed_date = datetime.date();

        Statement::try_from((&parsed_date, fmt))
    }
}

impl TryFrom<(&NaiveDate, &str)> for Statement {
    type Error = chrono::ParseError;

    fn try_from(value: (&NaiveDate, &str)) -> Result<Self, Self::Error> {
        let date = value.0;
        let fmt = value.1;

        let path = PathBuf::from(date.format(fmt).to_string());

        Ok(Statement::new(&path, date))
    }
}

impl TryFrom<(&Path, &str)> for Statement {
    type Error = chrono::ParseError;

    fn try_from(value: (&Path, &str)) -> Result<Self, Self::Error> {
        let path = value.0;
        let fmt = value.1;

        match NaiveDate::parse_from_str(path.file_name().unwrap().to_str().unwrap(), fmt) {
            Ok(date) => Ok(Statement::new(path, &date)),
            Err(e) => Err(e),
        }
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({:?})", self.date(), self.path())
    }
}

#[cfg(test)]
mod tests {
    use super::STATEMENT_DEFAULT_PATH_FMT;
    use crate::Statement;
    use chrono::NaiveDate;
    use std::{
        path::{Path, PathBuf},
        str::FromStr,
    };
    use toml::value::Datetime;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    fn check_try_from_path(input: (&Path, &str), expected: Result<Statement, chrono::ParseError>) {
        let observed = Statement::try_from(input);
        assert_eq!(expected, observed);
    }

    #[test]
    fn try_from_path_matching_format() {
        let input_path = PathBuf::from("2021-11-01.pdf");
        let input_fmt = "%Y-%m-%d.pdf";
        let expected_date = NaiveDate::from_ymd(2021, 11, 1);
        let expected = Statement::new(&input_path, &expected_date);

        check_try_from_path((&input_path, input_fmt), Ok(expected));
    }

    #[test]
    #[should_panic]
    fn try_from_path_mismatching_format() {
        let input_path = PathBuf::from("2021-11-01.pdf");
        let input_fmt = "not-the-right-format-%Y-%m-%d.pdf";
        let expected_date = NaiveDate::from_ymd(2021, 11, 1);
        let expected = Statement::new(&input_path, &expected_date);

        check_try_from_path((&input_path, input_fmt), Ok(expected));
    }

    fn check_from_naivedate(input: &NaiveDate, expected: Statement) {
        let observed = Statement::from(input);

        assert_eq!(expected, observed);
    }

    #[test]
    fn from_naivedate() {
        let date = NaiveDate::from_ymd(2021, 11, 21);
        let path = PathBuf::from(STATEMENT_DEFAULT_PATH_FMT);

        let expected = Statement { path, date };

        check_from_naivedate(&date, expected);
    }

    fn check_try_from_date(
        input: (&NaiveDate, &str),
        expected: Result<Statement, chrono::ParseError>,
    ) {
        let observed = Statement::try_from(input);
        assert_eq!(expected, observed);
    }

    #[test]
    fn try_from_date_matching_format() {
        let input_date = NaiveDate::from_ymd(2021, 11, 1);
        let input_fmt = "%Y-%m-%d.pdf";

        let expected_path = PathBuf::from("2021-11-01.pdf");
        let expected = Statement::new(&expected_path, &input_date);

        check_try_from_date((&input_date, input_fmt), Ok(expected));
    }

    #[test]
    #[should_panic]
    fn try_from_date_mismatching_format() {
        let input_date = NaiveDate::from_ymd(2021, 11, 1);
        let input_fmt = "%Y-.pdf";
        let expected_path = PathBuf::from("2021-11-01.pdf");
        let expected = Statement::new(&expected_path, &input_date);

        check_try_from_date((&input_date, input_fmt), Ok(expected));
    }

    fn try_check_from_datetime(
        input: (&Datetime, &str),
        expected: Result<Statement, chrono::ParseError>,
    ) {
        let observed = Statement::try_from(input);
        assert_eq!(expected, observed);
    }

    #[test]
    fn try_from_datetime_matching_format() {
        let input_datetime = Datetime::from_str("2021-11-01 00:00:00").unwrap();
        let input_fmt = "%Y-%m-%d.pdf";

        let expected_date = NaiveDate::from_ymd(2021, 11, 1);
        let expected_path = PathBuf::from("2021-11-01.pdf");
        let expected = Statement::new(&expected_path, &expected_date);

        try_check_from_datetime((&input_datetime, input_fmt), Ok(expected));
    }

    #[test]
    #[should_panic]
    fn try_from_datetime_mismatching_format() {
        let input_datetime = Datetime::from_str("2021-11-01 00:00:00").unwrap();
        let input_fmt = "%Y-.pdf";

        let expected_date = NaiveDate::from_ymd(2021, 11, 1);
        let expected_path = PathBuf::from("2021-11-01.pdf");
        let expected = Statement::new(&expected_path, &expected_date);

        try_check_from_datetime((&input_datetime, input_fmt), Ok(expected));
    }
}
