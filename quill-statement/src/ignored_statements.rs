//! A collection of ignored statements.

use chrono::{Local, NaiveDate};
use kronos::Shim;
use serde::Deserialize;
use std::{path::Path, slice::Iter};

// use crate::models::account::{expected_statement_dates, pair_dates_statements};

use crate::ignore_file::{ignorefile_path_from_dir, IgnoreFile};
use crate::{next_date_from_given, ObservedStatement, Statement, StatementStatus};

/// Control which account statements are ignored.
#[derive(Clone, Debug, Deserialize)]
pub struct IgnoredStatements {
    // statement dates that are being skipped/ignored
    stmts: Vec<Statement>,
}

impl IgnoredStatements {
    /// Construct an empty `IgnoredStatements` object.
    pub fn empty() -> Self {
        IgnoredStatements { stmts: vec![] }
    }

    /// Construct a new `IgnoredStatements` object.
    pub fn new<'a>(first: &NaiveDate, period: &Shim<'a>, fmt: &str, dir: &Path) -> Self {
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
mod tests {
    use super::*;

    #[track_caller]
    fn check_pair_dates_statements(
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
    fn empty_dates() {
        // Check all empty
        check_pair_dates_statements(&[], &[], &IgnoredStatements::empty(), vec![]);

        // Check non-empty input statements
        check_pair_dates_statements(
            &[],
            &[blank_statement(2021, 9, 22)],
            &IgnoredStatements::empty(),
            vec![],
        );

        // check non-empty ignored statements
        check_pair_dates_statements(
            &[],
            &[],
            &IgnoredStatements::from(vec![blank_statement(2021, 9, 22)]),
            vec![],
        );

        // Check non-empty, but non-overlapping, statements and ignores
        check_pair_dates_statements(
            &[],
            &[blank_statement(2021, 9, 22)],
            &IgnoredStatements::from(vec![blank_statement(2021, 10, 22)]),
            vec![],
        );

        // Check non-empty and overlapping statements and ignores
        check_pair_dates_statements(
            &[],
            &[blank_statement(2021, 9, 22)],
            &IgnoredStatements::from(vec![blank_statement(2021, 9, 22)]),
            vec![],
        );
    }

    #[test]
    /// Check that statements can be identified as missing
    fn missing() {
        // Check a single statement can be detected
        check_pair_dates_statements(
            &[NaiveDate::from_ymd(2021, 9, 22)],
            &[],
            &IgnoredStatements::empty(),
            vec![ObservedStatement::new(
                &blank_statement(2021, 9, 22),
                StatementStatus::Missing,
            )],
        );

        // Check that multiple statements can be detected
        check_pair_dates_statements(
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
    fn available() {
        // Check a single statement can be detected
        check_pair_dates_statements(
            &[NaiveDate::from_ymd(2021, 9, 22)],
            &[blank_statement(2021, 9, 22)],
            &IgnoredStatements::empty(),
            vec![ObservedStatement::new(
                &blank_statement(2021, 9, 22),
                StatementStatus::Available,
            )],
        );

        // Check multiple statements can be detected
        check_pair_dates_statements(
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

        check_pair_dates_statements(
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

        check_pair_dates_statements(
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

        check_pair_dates_statements(
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

        check_pair_dates_statements(
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

        check_pair_dates_statements(
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

        check_pair_dates_statements(
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
    fn ignore() {
        // Check that a single missing statement is properly ignored
        check_pair_dates_statements(
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

        check_pair_dates_statements(
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

        check_pair_dates_statements(
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
        check_pair_dates_statements(
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

        check_pair_dates_statements(
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

        check_pair_dates_statements(
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
        check_pair_dates_statements(
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
