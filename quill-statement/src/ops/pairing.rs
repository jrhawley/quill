//! Functions to pair dates with statements.

use chrono::{Local, NaiveDate};
use kronos::Shim;
use std::{path::Path, slice::Iter};

use crate::{
    next_date_from_given, statement_struct::STATEMENT_DEFAULT_PATH_FMT, IgnoredStatements,
    ObservedStatement, Statement, StatementStatus,
};

/// Given the past and future dates, move to a possible future date.
fn advance_to_next_dates<'a>(
    past_date: &mut &'a NaiveDate,
    fut_date: &'a NaiveDate,
    possible_fut_date: &mut Option<&'a NaiveDate>,
    date_iter: &mut Iter<'a, NaiveDate>,
) {
    *past_date = fut_date;
    *possible_fut_date = date_iter.next();
}

/// Advance to the next statement, if possible.
fn advance_to_next_statement<'a>(
    possible_stmt: &mut Option<&'a Statement>,
    stmt_iter: &mut Iter<'a, Statement>,
) {
    *possible_stmt = stmt_iter.next();
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

    // if there is no first date (i.e. first statement is in the future),
    // just return empty
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
                    Path::new(STATEMENT_DEFAULT_PATH_FMT),
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
                    Path::new(STATEMENT_DEFAULT_PATH_FMT),
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
                Path::new(STATEMENT_DEFAULT_PATH_FMT),
                StatementStatus::Missing,
                &mut pairs,
                &mut possible_ignore,
                &mut ignore_iter,
            );
            while let Some(fut_date) = possible_fut_date {
                pair_statement_with_date(
                    fut_date,
                    Path::new(STATEMENT_DEFAULT_PATH_FMT),
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
                Path::new(STATEMENT_DEFAULT_PATH_FMT),
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

fn pair_statement_with_date<'a>(
    expected_date: &NaiveDate,
    stmt_path: &Path,
    status: StatementStatus,
    target: &mut Vec<ObservedStatement>,
    possible_ignore: &mut Option<&'a NaiveDate>,
    ignore_iter: &mut Iter<'a, NaiveDate>,
) {
    let mut new_status = status;
    if let Some(ignored_date) = possible_ignore {
        if *expected_date == **ignored_date {
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
    possible_ignore: &mut Option<&'b NaiveDate>,
    ignore_iter: &mut Iter<'b, NaiveDate>,
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
    possible_ignore: &mut Option<&'b NaiveDate>,
    ignore_iter: &mut Iter<'b, NaiveDate>,
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
        expected: Vec<ObservedStatement>,
    ) {
        let observed = pair_dates_statements(input_dates, input_stmts, input_ignored);
        assert_eq!(expected, observed);
    }

    // A helper function for quickly created statments with a certain date
    fn blank_statement(year: i32, month: u32, day: u32) -> Statement {
        Statement::from(&NaiveDate::from_ymd(year, month, day))
    }

    #[test]
    fn empty_dates_empty_stmts_empty_ignore() {
        check_pair_dates_statements(&[], &[], &IgnoredStatements::empty(), vec![]);
    }

    #[test]
    fn empty_dates_one_stmt_empty_ignore() {
        check_pair_dates_statements(
            &[],
            &[blank_statement(2021, 9, 22)],
            &IgnoredStatements::empty(),
            vec![],
        );
    }

    #[test]
    fn empty_dates_empty_stmts_one_ignore() {
        check_pair_dates_statements(
            &[],
            &[],
            &IgnoredStatements::from(vec![NaiveDate::from_ymd(2021, 9, 22)]),
            vec![],
        );
    }

    #[test]
    fn empty_dates_overlapping_stmt_ignore() {
        check_pair_dates_statements(
            &[],
            &[blank_statement(2021, 9, 22)],
            &IgnoredStatements::from(vec![NaiveDate::from_ymd(2021, 10, 22)]),
            vec![],
        );
    }

    #[test]
    fn empty_dates_nonoverlapping_stmt_ignore() {
        check_pair_dates_statements(
            &[],
            &[blank_statement(2021, 9, 22)],
            &IgnoredStatements::from(vec![NaiveDate::from_ymd(2021, 9, 22)]),
            vec![],
        );
    }

    /// Check that a single statement can be detected as missing
    #[test]
    fn one_date_empty_stmts_empty_ignore() {
        let input_dates = &[NaiveDate::from_ymd(2021, 9, 22)];
        let input_stmts = &[];
        let input_ignored = &IgnoredStatements::empty();

        let expected = vec![ObservedStatement::new(
            &blank_statement(2021, 9, 22),
            StatementStatus::Missing,
        )];

        check_pair_dates_statements(input_dates, input_stmts, input_ignored, expected);
    }

    /// Check that multiple statements can be detected as missing
    #[test]
    fn multiple_dates_empty_stmts_empty_ignore() {
        let input_dates = &[
            NaiveDate::from_ymd(2021, 9, 22),
            NaiveDate::from_ymd(2021, 10, 22),
            NaiveDate::from_ymd(2021, 11, 22),
        ];
        let input_stmts = &[];
        let input_ignored = &IgnoredStatements::empty();

        let expected = vec![
            ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Missing),
            ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Missing),
            ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Missing),
        ];

        check_pair_dates_statements(input_dates, input_stmts, input_ignored, expected);
    }

    /// Check a single statement can be detected
    #[test]
    fn overlapping_one_date_one_stmt_empty_ignore() {
        let input_dates = &[NaiveDate::from_ymd(2021, 9, 22)];
        let input_stmts = &[blank_statement(2021, 9, 22)];
        let input_ignored = &IgnoredStatements::empty();

        let expected = vec![ObservedStatement::new(
            &blank_statement(2021, 9, 22),
            StatementStatus::Available,
        )];

        check_pair_dates_statements(input_dates, input_stmts, input_ignored, expected);
    }

    /// Check statements can be both missing and available
    #[test]
    fn first_avail_multiple_missing_empty_ignore() {
        let input_dates = &[
            NaiveDate::from_ymd(2021, 9, 22),
            NaiveDate::from_ymd(2021, 10, 22),
            NaiveDate::from_ymd(2021, 11, 22),
        ];
        let input_stmts = &[blank_statement(2021, 9, 22)];
        let input_ignored = &IgnoredStatements::empty();

        let expected = vec![
            ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Available),
            ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Missing),
            ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Missing),
        ];

        check_pair_dates_statements(input_dates, input_stmts, input_ignored, expected);
    }

    #[test]
    fn second_avail_multiple_missing_empty_ignore() {
        let input_dates = &[
            NaiveDate::from_ymd(2021, 9, 22),
            NaiveDate::from_ymd(2021, 10, 22),
            NaiveDate::from_ymd(2021, 11, 22),
        ];
        let input_stmts = &[blank_statement(2021, 10, 22)];
        let input_ignored = &IgnoredStatements::empty();

        let expected = vec![
            ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Missing),
            ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Available),
            ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Missing),
        ];

        check_pair_dates_statements(input_dates, input_stmts, input_ignored, expected);
    }

    #[test]
    fn third_avail_multiple_missing_empty_ignore() {
        let input_dates = &[
            NaiveDate::from_ymd(2021, 9, 22),
            NaiveDate::from_ymd(2021, 10, 22),
            NaiveDate::from_ymd(2021, 11, 22),
        ];
        let input_stmts = &[blank_statement(2021, 11, 22)];
        let input_ignored = &IgnoredStatements::empty();

        let expected = vec![
            ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Missing),
            ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Missing),
            ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Available),
        ];

        check_pair_dates_statements(input_dates, input_stmts, input_ignored, expected);
    }

    #[test]
    fn first_second_avail_one_missing_empty_ignore() {
        let input_dates = &[
            NaiveDate::from_ymd(2021, 9, 22),
            NaiveDate::from_ymd(2021, 10, 22),
            NaiveDate::from_ymd(2021, 11, 22),
        ];
        let input_stmts = &[blank_statement(2021, 9, 22), blank_statement(2021, 10, 22)];
        let input_ignored = &IgnoredStatements::empty();

        let expected = vec![
            ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Available),
            ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Available),
            ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Missing),
        ];

        check_pair_dates_statements(input_dates, input_stmts, input_ignored, expected);
    }

    #[test]
    fn first_third_avail_one_missing_empty_ignore() {
        let input_dates = &[
            NaiveDate::from_ymd(2021, 9, 22),
            NaiveDate::from_ymd(2021, 10, 22),
            NaiveDate::from_ymd(2021, 11, 22),
        ];
        let input_stmts = &[blank_statement(2021, 9, 22), blank_statement(2021, 10, 22)];
        let input_ignored = &IgnoredStatements::empty();

        let expected = vec![
            ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Available),
            ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Missing),
            ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Available),
        ];

        check_pair_dates_statements(input_dates, input_stmts, input_ignored, expected);
    }

    #[test]
    fn second_third_avail_one_missing_empty_ignore() {
        let input_dates = &[
            NaiveDate::from_ymd(2021, 9, 22),
            NaiveDate::from_ymd(2021, 10, 22),
            NaiveDate::from_ymd(2021, 11, 22),
        ];
        let input_stmts = &[blank_statement(2021, 10, 22), blank_statement(2021, 11, 22)];
        let input_ignored = &IgnoredStatements::empty();

        let expected = vec![
            ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Missing),
            ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Available),
            ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Available),
        ];

        check_pair_dates_statements(input_dates, input_stmts, input_ignored, expected);
    }

    #[test]
    fn all_avail_empty_ignore() {
        let input_dates = &[
            NaiveDate::from_ymd(2021, 9, 22),
            NaiveDate::from_ymd(2021, 10, 22),
            NaiveDate::from_ymd(2021, 11, 22),
        ];
        let input_stmts = &[
            blank_statement(2021, 9, 22),
            blank_statement(2021, 10, 22),
            blank_statement(2021, 11, 22),
        ];
        let input_ignored = &IgnoredStatements::empty();

        let expected = vec![
            ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Available),
            ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Available),
            ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Available),
        ];

        check_pair_dates_statements(input_dates, input_stmts, input_ignored, expected);
    }

    #[test]
    fn first_ignored_mutliple_missing() {
        let input_dates = &[
            NaiveDate::from_ymd(2021, 9, 22),
            NaiveDate::from_ymd(2021, 10, 22),
            NaiveDate::from_ymd(2021, 11, 22),
        ];
        let input_stmts = &[];
        let input_ignored = &IgnoredStatements::from(vec![NaiveDate::from_ymd(2021, 9, 22)]);

        let expected = vec![
            ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Ignored),
            ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Missing),
            ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Missing),
        ];

        check_pair_dates_statements(input_dates, input_stmts, input_ignored, expected);
    }

    #[test]
    fn second_ignored_mutliple_missing() {
        let input_dates = &[
            NaiveDate::from_ymd(2021, 9, 22),
            NaiveDate::from_ymd(2021, 10, 22),
            NaiveDate::from_ymd(2021, 11, 22),
        ];
        let input_stmts = &[];
        let input_ignored = &IgnoredStatements::from(vec![NaiveDate::from_ymd(2021, 10, 22)]);

        let expected = vec![
            ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Missing),
            ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Ignored),
            ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Missing),
        ];

        check_pair_dates_statements(input_dates, input_stmts, input_ignored, expected);
    }

    #[test]
    fn third_ignored_mutliple_missing() {
        let input_dates = &[
            NaiveDate::from_ymd(2021, 9, 22),
            NaiveDate::from_ymd(2021, 10, 22),
            NaiveDate::from_ymd(2021, 11, 22),
        ];
        let input_stmts = &[];
        let input_ignored = &IgnoredStatements::from(vec![NaiveDate::from_ymd(2021, 11, 22)]);

        let expected = vec![
            ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Missing),
            ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Missing),
            ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Ignored),
        ];

        check_pair_dates_statements(input_dates, input_stmts, input_ignored, expected);
    }

    #[test]
    fn first_second_ignored() {
        let input_dates = &[
            NaiveDate::from_ymd(2021, 9, 22),
            NaiveDate::from_ymd(2021, 10, 22),
            NaiveDate::from_ymd(2021, 11, 22),
        ];
        let input_stmts = &[];
        let input_ignored = &IgnoredStatements::from(vec![
            NaiveDate::from_ymd(2021, 9, 22),
            NaiveDate::from_ymd(2021, 10, 22),
        ]);

        let expected = vec![
            ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Ignored),
            ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Ignored),
            ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Missing),
        ];

        check_pair_dates_statements(input_dates, input_stmts, input_ignored, expected);
    }

    #[test]
    fn first_third_ignored() {
        let input_dates = &[
            NaiveDate::from_ymd(2021, 9, 22),
            NaiveDate::from_ymd(2021, 10, 22),
            NaiveDate::from_ymd(2021, 11, 22),
        ];
        let input_stmts = &[];
        let input_ignored = &IgnoredStatements::from(vec![
            NaiveDate::from_ymd(2021, 9, 22),
            NaiveDate::from_ymd(2021, 11, 22),
        ]);

        let expected = vec![
            ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Ignored),
            ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Missing),
            ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Ignored),
        ];

        check_pair_dates_statements(input_dates, input_stmts, input_ignored, expected);
    }

    #[test]
    fn second_third_ignored() {
        let input_dates = &[
            NaiveDate::from_ymd(2021, 9, 22),
            NaiveDate::from_ymd(2021, 10, 22),
            NaiveDate::from_ymd(2021, 11, 22),
        ];
        let input_stmts = &[];
        let input_ignored = &IgnoredStatements::from(vec![
            NaiveDate::from_ymd(2021, 10, 22),
            NaiveDate::from_ymd(2021, 11, 22),
        ]);

        let expected = vec![
            ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Missing),
            ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Ignored),
            ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Ignored),
        ];

        check_pair_dates_statements(input_dates, input_stmts, input_ignored, expected);
    }

    #[test]
    fn all_ignored() {
        let input_dates = &[
            NaiveDate::from_ymd(2021, 9, 22),
            NaiveDate::from_ymd(2021, 10, 22),
            NaiveDate::from_ymd(2021, 11, 22),
        ];
        let input_stmts = &[];
        let input_ignored = &IgnoredStatements::from(vec![
            NaiveDate::from_ymd(2021, 9, 22),
            NaiveDate::from_ymd(2021, 10, 22),
            NaiveDate::from_ymd(2021, 11, 22),
        ]);

        let expected = vec![
            ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Ignored),
            ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Ignored),
            ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Ignored),
        ];

        check_pair_dates_statements(input_dates, input_stmts, input_ignored, expected);
    }

    #[test]
    fn missing_ignored_available() {
        let input_dates = &[
            NaiveDate::from_ymd(2021, 9, 22),
            NaiveDate::from_ymd(2021, 10, 22),
            NaiveDate::from_ymd(2021, 11, 22),
        ];
        let input_stmts = &[Statement::from(&NaiveDate::from_ymd(2021, 9, 22))];
        let input_ignored = &IgnoredStatements::from(vec![NaiveDate::from_ymd(2021, 10, 22)]);

        let expected = vec![
            ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Available),
            ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Ignored),
            ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Missing),
        ];

        check_pair_dates_statements(input_dates, input_stmts, input_ignored, expected);
    }
}
