//! Information for a single account.

use chrono::prelude::*;
use kronos::Shim;
use quill_statement::{
    expected_statement_dates, next_date_from_given, next_date_from_today, pair_dates_statements,
    prev_date_from_given, prev_date_from_today, IgnoredStatements, ObservedStatement, Statement,
};
use std::convert::TryFrom;
use std::fmt::{Debug, Display};
use std::path::{Path, PathBuf};
use toml::Value;
use walkdir::WalkDir;

use super::parse::{
    parse_account_directory, parse_account_name, parse_first_statement_date,
    parse_institution_name, parse_statement_format, parse_statement_period,
};
use super::AccountCreationError;

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
        // all statements in the directory
        let files: Vec<PathBuf> = WalkDir::new(self.directory())
            .max_depth(1)
            .into_iter()
            .filter_map(|p| p.ok())
            .map(|p| p.into_path())
            .filter(|p| p.is_file())
            .collect();
        // dates from the statement names
        let mut stmts: Vec<Statement> = files
            .iter()
            .filter_map(|p| Statement::try_from((p.as_path(), self.statement_fmt.as_str())).ok())
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
        pair_dates_statements(&required, &available, self.ignored())
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
