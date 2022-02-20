//! Information for a single account.

use super::parse::{
    parse_account_directory, parse_account_name, parse_first_statement_date,
    parse_institution_name, parse_statement_format, parse_statement_period,
};
use super::AccountCreationError;
use chrono::prelude::*;
use kronos::Shim;
use quill_statement::{
    expected_statement_dates, next_date_from_given, next_date_from_today, pair_dates_statements,
    prev_date_from_given, prev_date_from_today, IgnoredStatements, ObservedStatement, Statement,
};
use regex::Regex;
use std::convert::TryFrom;
use std::ffi::OsStr;
use std::fmt::{Debug, Display};
use std::path::{Path, PathBuf};
use toml::Value;
use walkdir::WalkDir;

#[derive(Clone)]
/// Information related to an account, its billing period, and where to find the bills
pub struct Account<'a> {
    name: String,
    institution: String,
    statement_first: NaiveDate,
    statement_period: Shim<'a>,
    statement_fmt: String,
    dir: PathBuf,
    ignored: IgnoredStatements,
}

impl<'a> Account<'a> {
    /// Declare a new Account
    pub fn new(
        name: &str,
        institution: &str,
        first: NaiveDate,
        period: Shim<'a>,
        fmt: &str,
        dir: &Path,
    ) -> Account<'a> {
        Account {
            name: String::from(name),
            institution: String::from(institution),
            statement_first: first,
            statement_period: period,
            statement_fmt: String::from(fmt),
            dir: dir.to_path_buf(),
            ignored: IgnoredStatements::from(dir),
        }
    }

    /// Return the name of the account
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the account's first statement date
    pub fn first(&self) -> &NaiveDate {
        &self.statement_first
    }

    /// Return the name of the related institution
    pub fn institution(&self) -> &str {
        &self.institution
    }

    /// Return the directory containing statements for this account
    pub fn directory(&self) -> &Path {
        self.dir.as_path()
    }

    /// Return the directory containing statements for this account
    pub fn format_string(&self) -> &str {
        &self.statement_fmt
    }

    /// Return the ignored statements for this account
    pub fn ignored(&self) -> &IgnoredStatements {
        &self.ignored
    }

    /// Calculate the most recent statement before a given date for the account
    pub fn prev_statement_date(&self, date: NaiveDate) -> NaiveDate {
        prev_date_from_given(&date, &self.statement_period)
    }

    /// Print the most recent statement before today for the account
    pub fn prev_statement(&self) -> NaiveDate {
        prev_date_from_today(&self.statement_period)
    }

    /// Calculate the next statement for the account from a given date
    pub fn next_statement_date(&self, date: NaiveDate) -> NaiveDate {
        next_date_from_given(&date, &self.statement_period)
    }

    /// Print the next statement for the account from today
    pub fn next_statement(&self) -> NaiveDate {
        next_date_from_today(&self.statement_period)
    }

    /// List all statement dates for the account
    /// This list is guaranteed to be sorted, earliest first
    pub fn statement_dates(&self) -> Vec<NaiveDate> {
        expected_statement_dates(&self.statement_first, &self.statement_period)
    }

    /// Check the account's directory for all downloaded statements
    /// This list is guaranteed to be sorted, earliest first
    pub fn downloaded_statements(&self) -> Vec<Statement> {
        // all files in the directory
        let files: Vec<PathBuf> = WalkDir::new(self.directory())
            .max_depth(1)
            .into_iter()
            .filter_map(|p| p.ok())
            .map(|p| p.into_path())
            .filter(|p| p.is_file())
            .collect();

        // all files that match the statement format string
        let matching_files: Vec<PathBuf> = files
            .into_iter()
            .filter(|p| file_name_matches(p, self.format_string()))
            .collect();

        // a vec of the statements
        let mut stmts: Vec<Statement> = matching_files
            .iter()
            .filter_map(|p| Statement::try_from((p.as_path(), self.format_string())).ok())
            .collect();
        stmts.sort_by(|a, b| a.date().partial_cmp(b.date()).unwrap());

        stmts
    }

    /// Match expected and downloaded statements
    pub fn match_statements(&self) -> Vec<ObservedStatement> {
        // get expected statements
        let required = self.statement_dates();
        // get downloaded statements
        let available = self.downloaded_statements();

        // TODO: Fix
        match pair_dates_statements(&required, &available, self.ignored()) {
            Ok(v) => v,
            Err(_) => vec![],
        }
    }
}

impl<'a> Debug for Account<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.institution)
    }
}

impl<'a> Display for Account<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.institution)
    }
}

impl<'a> PartialEq<Account<'_>> for Account<'_> {
    fn eq(&self, other: &Account<'_>) -> bool {
        // TODO: Figure out what to do about the `statement_period` for equality
        (self.name() == other.name())
            && (self.first() == other.first())
            && (self.institution() == other.institution())
            && (self.directory() == other.directory())
            && (self.format_string() == other.format_string())
    }
}

impl<'a> TryFrom<&Value> for Account<'a> {
    type Error = AccountCreationError;

    fn try_from(props: &Value) -> Result<Self, Self::Error> {
        let name = parse_account_name(props)?;
        let institution = parse_institution_name(props)?;
        let fmt = parse_statement_format(props)?;
        let dir_buf = parse_account_directory(props)?;
        let dir = dir_buf.as_path();
        let first = parse_first_statement_date(props)?;
        let period = parse_statement_period(props)?;

        Ok(Account::new(name, institution, first, period, fmt, dir))
    }
}

/// Check if the path's filename matches a given regex
fn file_name_matches(path: &Path, fmt: &str) -> bool {
    let fname = path
        .file_name()
        .unwrap_or(OsStr::new(""))
        .to_str()
        .unwrap_or("");

    // extract the date, if possible, from the file name with the statement's
    // format string
    let fname_date = match NaiveDate::parse_from_str(fname, fmt) {
        Ok(d) => d,
        Err(_) => return false,
    };

    // reconstruct what the filename for this date should be
    let re_str = format!("^{}$", fname_date.format(fmt));
    let re = Regex::new(&re_str).unwrap();

    // check for the match
    let matching = re.is_match(fname);

    matching
}

#[cfg(test)]
mod tests {
    use super::*;
    use kronos::{Grain, Grains, NthOf};

    #[track_caller]
    fn check_new(input: (&str, &str, NaiveDate, Shim<'static>, &str, &Path), expected: Account) {
        let observed = Account::new(input.0, input.1, input.2, input.3, input.4, input.5);

        assert_eq!(expected, observed);
    }

    #[test]
    fn new() {
        let input = (
            "test name",
            "institution name",
            NaiveDate::from_ymd(2011, 1, 1),
            Shim::new(NthOf(1, Grains(Grain::Day), Grains(Grain::Month))),
            "%Y-%m-%d.pdf",
            Path::new("test-dir"),
        );
        let expected = Account {
            name: "test name".to_string(),
            institution: "institution name".to_string(),
            statement_first: NaiveDate::from_ymd(2011, 1, 1),
            statement_period: Shim::new(NthOf(1, Grains(Grain::Day), Grains(Grain::Month))),
            statement_fmt: "%Y-%m-%d.pdf".to_string(),
            dir: PathBuf::from("test-dir"),
            ignored: IgnoredStatements::empty(),
        };

        check_new(input, expected);
    }

    #[track_caller]
    fn check_file_name_matches(input: (&Path, &str), expected: bool) {
        let observed = file_name_matches(input.0, input.1);

        assert_eq!(expected, observed)
    }

    #[test]
    fn simple_format() {
        let path = Path::new("2021-01-01.pdf");
        let s = "%Y-%m-%d.pdf";

        check_file_name_matches((path, s), true);
    }

    #[test]
    fn simple_format_nonmatching() {
        let path = Path::new("2021-01-01 other file with text.pdf");
        let s = "%Y-%m-%d.pdf";

        check_file_name_matches((path, s), false);
    }

    #[test]
    fn downloaded_none() {
        let acct = Account::new(
            "Name",
            "Institution",
            NaiveDate::from_ymd(2021, 1, 1),
            Shim::new(NthOf(1, Grains(Grain::Day), Grains(Grain::Month))),
            "%Y-%m-%d.pdf",
            Path::new("tests/no-statements"),
        );
        let expected: Vec<Statement> = vec![];

        assert_eq!(expected, acct.downloaded_statements());
    }

    #[test]
    fn downloaded_some() {
        let acct = Account::new(
            "Name",
            "Institution",
            NaiveDate::from_ymd(2021, 1, 1),
            Shim::new(NthOf(1, Grains(Grain::Day), Grains(Grain::Month))),
            "%Y-%m-%d.pdf",
            Path::new("tests/exact-matching-statements"),
        );

        let expected = vec![
            Statement::new(
                Path::new("tests/exact-matching-statements/2021-01-01.pdf"),
                &NaiveDate::from_ymd(2021, 1, 1),
            ),
            Statement::new(
                Path::new("tests/exact-matching-statements/2021-02-01.pdf"),
                &NaiveDate::from_ymd(2021, 2, 1),
            ),
        ];

        assert_eq!(expected, acct.downloaded_statements());
    }

    #[test]
    fn downloaded_some_with_others() {
        let acct = Account::new(
            "Name",
            "Institution",
            NaiveDate::from_ymd(2021, 1, 1),
            Shim::new(NthOf(1, Grains(Grain::Day), Grains(Grain::Month))),
            "%Y-%m-%d.pdf",
            Path::new("tests/matching-with-others"),
        );

        let expected = vec![
            Statement::new(
                Path::new("tests/matching-with-others/2021-01-01.pdf"),
                &NaiveDate::from_ymd(2021, 1, 1),
            ),
            Statement::new(
                Path::new("tests/matching-with-others/2021-02-01.pdf"),
                &NaiveDate::from_ymd(2021, 2, 1),
            ),
        ];

        assert_eq!(expected, acct.downloaded_statements());
    }
}
