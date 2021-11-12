//! Information for a single account.

use chrono::prelude::*;
use chrono::Duration;
use kronos::{Shim, TimeSequence};
use log::warn;
use std::convert::TryFrom;
use std::fmt::{Debug, Display};
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::slice::Iter;
use toml::Value;
use walkdir::WalkDir;

use super::{Date, ObservedStatement, Statement, StatementStatus};
use super::date::next_weekday_date;
use super::ignore::IgnoredStatements;
use super::parse::{
    parse_account_directory, parse_account_name, parse_first_statement_date,
    parse_institution_name, parse_statement_format, parse_statement_period,
};

/// File within the account's directory that lists what statement dates
/// should be ignored.
const IGNOREFILE: &str = ".quillignore.toml";

#[derive(Clone)]
/// Information related to an account, its billing period, and where to find the bills
pub struct Account<'a> {
    name: String,
    institution: String,
    statement_first: Date,
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
        first: Date,
        period: Shim<'a>,
        fmt: &str,
        dir: &Path,
    ) -> Account<'a> {
        // print warning if the directory cannot be found
        if !dir.exists() {
            warn!("Account `{}` with directory `{}` cannot be found. Statements may not be processed properly.", name, dir.display());
        }

        let ig_stmts = IgnoredStatements::new(&first, &period, fmt, dir);

        Account {
            name: String::from(name),
            institution: String::from(institution),
            statement_first: first,
            statement_period: period,
            statement_fmt: String::from(fmt),
            dir: dir.to_path_buf(),
            ignored: ig_stmts,
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
    pub fn prev_statement_date(&self, date: Date) -> Date {
        prev_date_from_given(&date, &self.statement_period)
    }

    /// Print the most recent statement before today for the account
    pub fn prev_statement(&self) -> Date {
        prev_date_from_today(&self.statement_period)
    }

    /// Calculate the next statement for the account from a given date
    pub fn next_statement_date(&self, date: Date) -> Date {
        next_date_from_given(&date, &self.statement_period)
    }

    /// Print the next statement for the account from today
    pub fn next_statement(&self) -> Date {
        next_date_from_today(&self.statement_period)
    }

    /// List all statement dates for the account
    /// This list is guaranteed to be sorted, earliest first
    pub fn statement_dates(&self) -> Vec<Date> {
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
            .filter_map(|p| Statement::from_path(p, &self.statement_fmt).ok())
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
    type Error = io::Error;
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

/// Match elements of Dates and Statements together to find closest pairing.
/// Finds a 1:1 mapping of dates to statements, if possible.
pub fn pair_dates_statements(
    dates: &[Date],
    stmts: &[Statement],
    ignored: &IgnoredStatements,
) -> Vec<ObservedStatement> {
    // iterators over sorted dates
    let mut date_iter = dates.iter();
    let mut stmt_iter = stmts.iter();
    let mut ignore_iter = ignored.iter();
    let mut pairs: Vec<ObservedStatement> = vec![];

    // iteration placeholders
    // if there is no first statement required
    // (i.e. first statement is in the future), then just return empty
    let mut past_date = match date_iter.next() {
        Some(d) => d,
        None => return vec![],
    };
    let mut possible_fut_date = date_iter.next();
    let mut possible_stmt = stmt_iter.next();
    let mut possible_ignore = ignore_iter.next();
    let mut is_past_paired = false;

    // walk over the pair of dates and each statement
    // loop exits when there are either no more dates or no more statements to consider
    while let (Some(fut_date), Some(stmt)) = (possible_fut_date, possible_stmt) {
        if stmt.date() == past_date {
            // if the statement's date is equal to the earlier date being considered
            pair_statement_with_past(past_date, &mut is_past_paired, &mut possible_stmt, &mut stmt_iter, stmt.path(), &mut pairs, &possible_ignore);
        } else if stmt.date() == fut_date {
            // if the statement's date is equal to the later date being considered
            pair_statement_with_future(fut_date, &mut is_past_paired, &mut possible_stmt, &mut stmt_iter, stmt.path(), &mut pairs, &possible_ignore);
        } else if stmt.date() < fut_date {
            // if the statement is in between the past and future dates
            if !is_past_paired {
                // pair the statement with the past date if the past date doesn't currently have a statement paired with it
                pair_statement_with_past(past_date, &mut is_past_paired, &mut possible_stmt, &mut stmt_iter, stmt.path(), &mut pairs, &possible_ignore);
            } else {
                // if the past date has been paired up already, pair this statement with the future date
                pair_statement_with_future(fut_date, &mut is_past_paired, &mut possible_stmt, &mut stmt_iter, stmt.path(), &mut pairs, &possible_ignore);
            }
        } else {
            // if the statement is further ahead than the future date
            if !is_past_paired {
                // if the past date still hasn't been paired up, set it as missing
                pair_statement_with_date(past_date, Path::new(""), StatementStatus::Missing, &mut pairs, &possible_ignore);
            }

            // leave it to the next iteration to decide where the statement should be matched
            is_past_paired = false;
        }

        // each iteration always advances the dates forward, regardless of if either of them are paired with
        advance_to_next_dates(&mut past_date, fut_date, &mut possible_fut_date, &mut date_iter, &mut possible_ignore, &mut ignore_iter);
    }

    // if there are no more statements, then every remaining date should be counted as missing
    match (possible_fut_date, possible_stmt, is_past_paired) {
        (Some(fut_date), None, true) => {
            pair_statement_with_date(fut_date, Path::new(""), StatementStatus::Missing, &mut pairs, &possible_ignore);
            while let Some(fut_date) = possible_fut_date {
                pair_statement_with_date(fut_date, Path::new(""), StatementStatus::Missing, &mut pairs, &possible_ignore);
                advance_to_next_dates(&mut past_date, fut_date, &mut possible_fut_date, &mut date_iter, &mut possible_ignore, &mut ignore_iter);
            }
        }
        (Some(_), None, false) => {
            pair_statement_with_date(past_date, Path::new(""), StatementStatus::Missing, &mut pairs, &possible_ignore);
            while let Some(fut_date) = possible_fut_date {
                pair_statement_with_date(fut_date, Path::new(""), StatementStatus::Missing, &mut pairs, &possible_ignore);
                advance_to_next_dates(&mut past_date, fut_date, &mut possible_fut_date, &mut date_iter, &mut possible_ignore, &mut ignore_iter);
            }
        }
        (None, Some(stmt), false) => {
            pair_statement_with_date(past_date, stmt.path(), StatementStatus::Available, &mut pairs, &possible_ignore);
        }
        (None, None, false) => {
            pair_statement_with_date(past_date, Path::new(""), StatementStatus::Missing, &mut pairs, &possible_ignore);
        }
        (_, _, _) => {}
    }

    pairs
}

fn advance_to_next_dates<'a, 'b>(past_date: &mut &'a Date, fut_date: &'a Date, possible_fut_date: &mut Option<&'a Date>, date_iter: &mut Iter<'a, Date>, possible_ignore: &mut Option<&'b Statement>, ignore_iter: &mut Iter<'b, Statement>) {
    if let Some(ignored_stmt) = possible_ignore {
        if ignored_stmt.date() <= past_date {
            *possible_ignore = ignore_iter.next();
        }
    }
    *past_date = fut_date;
    *possible_fut_date = date_iter.next();
}

fn advance_to_next_statement<'a>(possible_stmt: &mut Option<&'a Statement>, stmt_iter: &mut Iter<'a, Statement>) {
    *possible_stmt = stmt_iter.next();
}

fn pair_statement_with_date(expected_date: &Date, stmt_path: &Path, status: StatementStatus, target: &mut Vec<ObservedStatement>, possible_ignore: &Option<&Statement>) {
    let mut new_status = status;
    if let Some(ignored_stmt) = possible_ignore {
        if expected_date == ignored_stmt.date() {
            new_status = StatementStatus::Ignored;
        }
    }
    let paired_stmt = Statement::new(stmt_path, expected_date);
    let paired_obs_stmt = ObservedStatement::new(&paired_stmt, new_status);
    target.push(paired_obs_stmt);
}

fn pair_statement_with_past<'a>(past_date: &Date, is_past_paired: &mut bool, possible_stmt: &mut Option<&'a Statement>, stmt_iter: &mut Iter<'a, Statement>, stmt_path: &Path, target: &mut Vec<ObservedStatement>, possible_ignore: &Option<&Statement>) {
    pair_statement_with_date(past_date, stmt_path, StatementStatus::Available, target, possible_ignore);
    *is_past_paired = false;
    advance_to_next_statement(possible_stmt, stmt_iter);
}

fn pair_statement_with_future<'a>(fut_date: &Date, is_past_paired: &mut bool, possible_stmt: &mut Option<&'a Statement>, stmt_iter: &mut Iter<'a, Statement>, stmt_path: &Path, target: &mut Vec<ObservedStatement>, possible_ignore: &Option<&Statement>) {
    pair_statement_with_date(fut_date, stmt_path, StatementStatus::Available, target, possible_ignore);
    *is_past_paired = true;
    advance_to_next_statement(possible_stmt, stmt_iter);
}

/// List all statement dates given a first date and period
/// This list is guaranteed to be sorted, earliest first
pub fn expected_statement_dates<'a>(first: &Date, period: &Shim<'a>) -> Vec<Date> {
    // statement Dates to be returned
    let mut stmnts = Vec::new();
    let now = Date(Local::today().naive_local());
    // add the first statement date if it is earlier than today
    if *first <= now {
        stmnts.push((*first).clone());
    }

    // iterate through all future statement dates
    let mut iter_date = next_date_from_given(first, period);
    while iter_date <= now {
        stmnts.push(iter_date);
        // get the next date after the current iterated date
        iter_date = next_date_from_given(&iter_date, period);
    }
    stmnts.sort();

    stmnts
}

/// Calculate the next periodic date starting from a given date.
pub fn next_date_from_given<'a>(from: &Date, period: &Shim<'a>) -> Date {
    // need to shift date  by one day, because of how future is called
    let d = period
        .future(&(from.0 + Duration::days(1)).and_hms(0, 0, 0))
        .next()
        .unwrap()
        .start
        .date();
    // adjust for weekends
    // still adding days since statements are typically released after weekends, not before
    next_weekday_date(d)
}

/// Calculate the next periodic date starting from today.
pub fn next_date_from_today<'a>(period: &Shim<'a>) -> Date {
    let today = Date(Local::now().naive_local().date());
    next_date_from_given(&today, period)
}

/// Calculate the most recent periodic date before a given date.
pub fn prev_date_from_given<'a>(from: &Date, period: &Shim<'a>) -> Date {
    // find the next statement
    let d = period
        .past(&from.and_hms(0, 0, 0))
        .next()
        .unwrap()
        .start
        .date();
    // adjust for weekends
    // still adding days since statements are typically released after weekends, not before
    next_weekday_date(d)
}

/// Calculate the most recent periodic date before today
pub fn prev_date_from_today<'a>(period: &Shim<'a>) -> Date {
    let today = Date(Local::now().naive_local().date());
    prev_date_from_given(&today, period)
}
