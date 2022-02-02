//! Information for a single account.

use chrono::prelude::*;
use kronos::Shim;
use log::warn;
use quill_statement::{
    next_date_from_given, next_date_from_today, prev_date_from_given, prev_date_from_today,
    IgnoredStatements, ObservedStatement, Statement, StatementStatus,
};
use std::convert::TryFrom;
use std::fmt::{Debug, Display};
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::slice::Iter;
use toml::Value;
use walkdir::WalkDir;

use super::parse::{
    parse_account_directory, parse_account_name, parse_first_statement_date,
    parse_institution_name, parse_statement_format, parse_statement_period,
};

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
    dates: &[NaiveDate],
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
            pair_statement_with_past(
                past_date,
                &mut is_past_paired,
                &mut possible_stmt,
                &mut stmt_iter,
                stmt.path(),
                &mut pairs,
                &mut possible_ignore,
                &mut ignore_iter,
            );
        } else if stmt.date() == fut_date {
            // if the statement's date is equal to the later date being considered
            pair_statement_with_future(
                past_date,
                fut_date,
                &mut is_past_paired,
                &mut possible_stmt,
                &mut stmt_iter,
                stmt.path(),
                &mut pairs,
                &mut possible_ignore,
                &mut ignore_iter,
            );
        } else if stmt.date() < fut_date {
            // if the statement is in between the past and future dates
            let dist_to_past = *stmt.date() - *past_date;
            let dist_to_fut = *fut_date - *stmt.date();

            if !is_past_paired && (dist_to_past < dist_to_fut) {
                // pair the statement with the past date if the past date
                // doesn't currently have a statement paired with it and
                // the statement is closer to it
                pair_statement_with_past(
                    past_date,
                    &mut is_past_paired,
                    &mut possible_stmt,
                    &mut stmt_iter,
                    stmt.path(),
                    &mut pairs,
                    &mut possible_ignore,
                    &mut ignore_iter,
                );
            } else {
                // if the past date has been paired up already, pair this statement with the future date
                pair_statement_with_future(
                    past_date,
                    fut_date,
                    &mut is_past_paired,
                    &mut possible_stmt,
                    &mut stmt_iter,
                    stmt.path(),
                    &mut pairs,
                    &mut possible_ignore,
                    &mut ignore_iter,
                );
            }
        } else {
            // if the statement is further ahead than the future date
            if !is_past_paired {
                // if the past date still hasn't been paired up, set it as missing
                pair_statement_with_date(
                    past_date,
                    Path::new(""),
                    StatementStatus::Missing,
                    &mut pairs,
                    &mut possible_ignore,
                    &mut ignore_iter,
                );
            }

            // leave it to the next iteration to decide where the statement should be matched
            is_past_paired = false;
        }

        // each iteration always advances the dates forward, regardless of if either of them are paired with
        advance_to_next_dates(
            &mut past_date,
            fut_date,
            &mut possible_fut_date,
            &mut date_iter,
        );
    }

    // if there are no more statements, then every remaining date should be counted as missing
    match (possible_fut_date, possible_stmt, is_past_paired) {
        (Some(_), None, true) => {
            while let Some(fut_date) = possible_fut_date {
                pair_statement_with_date(
                    fut_date,
                    Path::new(""),
                    StatementStatus::Missing,
                    &mut pairs,
                    &mut possible_ignore,
                    &mut ignore_iter,
                );
                advance_to_next_dates(
                    &mut past_date,
                    fut_date,
                    &mut possible_fut_date,
                    &mut date_iter,
                );
            }
        }
        (Some(_), None, false) => {
            pair_statement_with_date(
                past_date,
                Path::new(""),
                StatementStatus::Missing,
                &mut pairs,
                &mut possible_ignore,
                &mut ignore_iter,
            );
            while let Some(fut_date) = possible_fut_date {
                pair_statement_with_date(
                    fut_date,
                    Path::new(""),
                    StatementStatus::Missing,
                    &mut pairs,
                    &mut possible_ignore,
                    &mut ignore_iter,
                );
                advance_to_next_dates(
                    &mut past_date,
                    fut_date,
                    &mut possible_fut_date,
                    &mut date_iter,
                );
            }
        }
        (None, Some(stmt), false) => {
            pair_statement_with_date(
                past_date,
                stmt.path(),
                StatementStatus::Available,
                &mut pairs,
                &mut possible_ignore,
                &mut ignore_iter,
            );
        }
        (None, None, false) => {
            pair_statement_with_date(
                past_date,
                Path::new(""),
                StatementStatus::Missing,
                &mut pairs,
                &mut possible_ignore,
                &mut ignore_iter,
            );
        }
        (_, _, _) => {}
    }

    pairs
}

fn advance_to_next_dates<'a>(
    past_date: &mut &'a NaiveDate,
    fut_date: &'a NaiveDate,
    possible_fut_date: &mut Option<&'a NaiveDate>,
    date_iter: &mut Iter<'a, NaiveDate>,
) {
    *past_date = fut_date;
    *possible_fut_date = date_iter.next();
}

fn advance_to_next_statement<'a>(
    possible_stmt: &mut Option<&'a Statement>,
    stmt_iter: &mut Iter<'a, Statement>,
) {
    *possible_stmt = stmt_iter.next();
}

fn pair_statement_with_date<'a>(
    expected_date: &NaiveDate,
    stmt_path: &Path,
    status: StatementStatus,
    target: &mut Vec<ObservedStatement>,
    possible_ignore: &mut Option<&'a Statement>,
    ignore_iter: &mut Iter<'a, Statement>,
) {
    let mut new_status = status;
    if let Some(ignored_stmt) = possible_ignore {
        if *expected_date == *ignored_stmt.date() {
            new_status = StatementStatus::Ignored;
            // we've ignored this statement, we can move onto the next possible ignored statement
            *possible_ignore = ignore_iter.next();
        }
    }
    let paired_stmt = Statement::new(stmt_path, expected_date);
    let paired_obs_stmt = ObservedStatement::new(&paired_stmt, new_status);
    target.push(paired_obs_stmt);
}

fn pair_statement_with_past<'a, 'b>(
    past_date: &NaiveDate,
    is_past_paired: &mut bool,
    possible_stmt: &mut Option<&'a Statement>,
    stmt_iter: &mut Iter<'a, Statement>,
    stmt_path: &Path,
    target: &mut Vec<ObservedStatement>,
    possible_ignore: &mut Option<&'b Statement>,
    ignore_iter: &mut Iter<'b, Statement>,
) {
    pair_statement_with_date(
        past_date,
        stmt_path,
        StatementStatus::Available,
        target,
        possible_ignore,
        ignore_iter,
    );
    *is_past_paired = false;
    advance_to_next_statement(possible_stmt, stmt_iter);
}

fn pair_statement_with_future<'a, 'b>(
    past_date: &NaiveDate,
    fut_date: &NaiveDate,
    is_past_paired: &mut bool,
    possible_stmt: &mut Option<&'a Statement>,
    stmt_iter: &mut Iter<'a, Statement>,
    stmt_path: &Path,
    target: &mut Vec<ObservedStatement>,
    possible_ignore: &mut Option<&'b Statement>,
    ignore_iter: &mut Iter<'b, Statement>,
) {
    if !(*is_past_paired) {
        // assigning to the future without assigning to the past means that the past date is missing
        pair_statement_with_date(
            past_date,
            stmt_path,
            StatementStatus::Missing,
            target,
            possible_ignore,
            ignore_iter,
        );
    }

    pair_statement_with_date(
        fut_date,
        stmt_path,
        StatementStatus::Available,
        target,
        possible_ignore,
        ignore_iter,
    );
    *is_past_paired = true;
    advance_to_next_statement(possible_stmt, stmt_iter);
}

/// List all statement dates given a first date and period
/// This list is guaranteed to be sorted, earliest first
pub fn expected_statement_dates<'a>(first: &NaiveDate, period: &Shim<'a>) -> Vec<NaiveDate> {
    // statement Dates to be returned
    let mut stmnts = Vec::new();
    let now = Local::today().naive_local();
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

#[cfg(test)]
mod tests_pair_dates_statements {
    use super::*;

    #[track_caller]
    fn check(
        input_dates: &[NaiveDate],
        input_stmts: &[Statement],
        input_ignored: &IgnoredStatements,
        expected_result: Vec<ObservedStatement>,
    ) {
        let observed_result = pair_dates_statements(input_dates, input_stmts, input_ignored);
        assert_eq!(expected_result, observed_result);
    }

    fn blank_statement(year: i32, month: u32, day: u32) -> Statement {
        Statement::new(Path::new(""), &NaiveDate::from_ymd(year, month, day))
    }

    #[test]
    /// Check that empty dates returns an empty vec, regardless of the other
    /// arguments.
    fn test_empty_dates() {
        // Check all empty
        check(&[], &[], &IgnoredStatements::empty(), vec![]);

        // Check non-empty input statements
        check(
            &[],
            &[blank_statement(2021, 9, 22)],
            &IgnoredStatements::empty(),
            vec![],
        );

        // check non-empty ignored statements
        check(
            &[],
            &[],
            &IgnoredStatements::from(vec![blank_statement(2021, 9, 22)]),
            vec![],
        );

        // Check non-empty, but non-overlapping, statements and ignores
        check(
            &[],
            &[blank_statement(2021, 9, 22)],
            &IgnoredStatements::from(vec![blank_statement(2021, 10, 22)]),
            vec![],
        );

        // Check non-empty and overlapping statements and ignores
        check(
            &[],
            &[blank_statement(2021, 9, 22)],
            &IgnoredStatements::from(vec![blank_statement(2021, 9, 22)]),
            vec![],
        );
    }

    #[test]
    /// Check that statements can be identified as missing
    fn test_missing() {
        // Check a single statement can be detected
        check(
            &[NaiveDate::from_ymd(2021, 9, 22)],
            &[],
            &IgnoredStatements::empty(),
            vec![ObservedStatement::new(
                &blank_statement(2021, 9, 22),
                StatementStatus::Missing,
            )],
        );

        // Check that multiple statements can be detected
        check(
            &[
                NaiveDate::from_ymd(2021, 9, 22),
                NaiveDate::from_ymd(2021, 10, 22),
                NaiveDate::from_ymd(2021, 11, 22),
            ],
            &[],
            &IgnoredStatements::empty(),
            vec![
                ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Missing),
                ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Missing),
                ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Missing),
            ],
        );
    }

    #[test]
    /// Check that statements can be detected as available
    fn test_available() {
        // Check a single statement can be detected
        check(
            &[NaiveDate::from_ymd(2021, 9, 22)],
            &[blank_statement(2021, 9, 22)],
            &IgnoredStatements::empty(),
            vec![ObservedStatement::new(
                &blank_statement(2021, 9, 22),
                StatementStatus::Available,
            )],
        );

        // Check multiple statements can be detected
        check(
            &[
                NaiveDate::from_ymd(2021, 9, 22),
                NaiveDate::from_ymd(2021, 10, 22),
                NaiveDate::from_ymd(2021, 11, 22),
            ],
            &[blank_statement(2021, 9, 22)],
            &IgnoredStatements::empty(),
            vec![
                ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Available),
                ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Missing),
                ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Missing),
            ],
        );

        check(
            &[
                NaiveDate::from_ymd(2021, 9, 22),
                NaiveDate::from_ymd(2021, 10, 22),
                NaiveDate::from_ymd(2021, 11, 22),
            ],
            &[blank_statement(2021, 10, 22)],
            &IgnoredStatements::empty(),
            vec![
                ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Missing),
                ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Available),
                ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Missing),
            ],
        );

        check(
            &[
                NaiveDate::from_ymd(2021, 9, 22),
                NaiveDate::from_ymd(2021, 10, 22),
                NaiveDate::from_ymd(2021, 11, 22),
            ],
            &[blank_statement(2021, 11, 22)],
            &IgnoredStatements::empty(),
            vec![
                ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Missing),
                ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Missing),
                ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Available),
            ],
        );

        check(
            &[
                NaiveDate::from_ymd(2021, 9, 22),
                NaiveDate::from_ymd(2021, 10, 22),
                NaiveDate::from_ymd(2021, 11, 22),
            ],
            &[blank_statement(2021, 9, 22), blank_statement(2021, 10, 22)],
            &IgnoredStatements::empty(),
            vec![
                ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Available),
                ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Available),
                ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Missing),
            ],
        );

        check(
            &[
                NaiveDate::from_ymd(2021, 9, 22),
                NaiveDate::from_ymd(2021, 10, 22),
                NaiveDate::from_ymd(2021, 11, 22),
            ],
            &[blank_statement(2021, 9, 22), blank_statement(2021, 11, 22)],
            &IgnoredStatements::empty(),
            vec![
                ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Available),
                ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Missing),
                ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Available),
            ],
        );

        check(
            &[
                NaiveDate::from_ymd(2021, 9, 22),
                NaiveDate::from_ymd(2021, 10, 22),
                NaiveDate::from_ymd(2021, 11, 22),
            ],
            &[blank_statement(2021, 10, 22), blank_statement(2021, 11, 22)],
            &IgnoredStatements::empty(),
            vec![
                ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Missing),
                ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Available),
                ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Available),
            ],
        );

        check(
            &[
                NaiveDate::from_ymd(2021, 9, 22),
                NaiveDate::from_ymd(2021, 10, 22),
                NaiveDate::from_ymd(2021, 11, 22),
            ],
            &[
                blank_statement(2021, 9, 22),
                blank_statement(2021, 10, 22),
                blank_statement(2021, 11, 22),
            ],
            &IgnoredStatements::empty(),
            vec![
                ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Available),
                ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Available),
                ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Available),
            ],
        );
    }

    #[test]
    /// Check that no statements means all dates are determined as missing, unless ignored
    fn test_ignore() {
        // Check that a single missing statement is properly ignored
        check(
            &[
                NaiveDate::from_ymd(2021, 9, 22),
                NaiveDate::from_ymd(2021, 10, 22),
                NaiveDate::from_ymd(2021, 11, 22),
            ],
            &[],
            &IgnoredStatements::from(vec![blank_statement(2021, 9, 22)]),
            vec![
                ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Ignored),
                ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Missing),
                ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Missing),
            ],
        );

        check(
            &[
                NaiveDate::from_ymd(2021, 9, 22),
                NaiveDate::from_ymd(2021, 10, 22),
                NaiveDate::from_ymd(2021, 11, 22),
            ],
            &[],
            &IgnoredStatements::from(vec![blank_statement(2021, 10, 22)]),
            vec![
                ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Missing),
                ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Ignored),
                ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Missing),
            ],
        );

        check(
            &[
                NaiveDate::from_ymd(2021, 9, 22),
                NaiveDate::from_ymd(2021, 10, 22),
                NaiveDate::from_ymd(2021, 11, 22),
            ],
            &[],
            &IgnoredStatements::from(vec![blank_statement(2021, 11, 22)]),
            vec![
                ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Missing),
                ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Missing),
                ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Ignored),
            ],
        );

        // Check that multiple missing statements are properly ignored
        check(
            &[
                NaiveDate::from_ymd(2021, 9, 22),
                NaiveDate::from_ymd(2021, 10, 22),
                NaiveDate::from_ymd(2021, 11, 22),
            ],
            &[],
            &IgnoredStatements::from(vec![
                blank_statement(2021, 9, 22),
                blank_statement(2021, 10, 22),
            ]),
            vec![
                ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Ignored),
                ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Ignored),
                ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Missing),
            ],
        );

        check(
            &[
                NaiveDate::from_ymd(2021, 9, 22),
                NaiveDate::from_ymd(2021, 10, 22),
                NaiveDate::from_ymd(2021, 11, 22),
            ],
            &[],
            &IgnoredStatements::from(vec![
                blank_statement(2021, 9, 22),
                blank_statement(2021, 11, 22),
            ]),
            vec![
                ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Ignored),
                ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Missing),
                ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Ignored),
            ],
        );

        check(
            &[
                NaiveDate::from_ymd(2021, 9, 22),
                NaiveDate::from_ymd(2021, 10, 22),
                NaiveDate::from_ymd(2021, 11, 22),
            ],
            &[],
            &IgnoredStatements::from(vec![
                blank_statement(2021, 10, 22),
                blank_statement(2021, 11, 22),
            ]),
            vec![
                ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Missing),
                ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Ignored),
                ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Ignored),
            ],
        );

        // Check that all statements are properly ignored
        check(
            &[
                NaiveDate::from_ymd(2021, 9, 22),
                NaiveDate::from_ymd(2021, 10, 22),
                NaiveDate::from_ymd(2021, 11, 22),
            ],
            &[],
            &IgnoredStatements::from(vec![
                blank_statement(2021, 9, 22),
                blank_statement(2021, 10, 22),
                blank_statement(2021, 11, 22),
            ]),
            vec![
                ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Ignored),
                ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Ignored),
                ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Ignored),
            ],
        );
    }
}
