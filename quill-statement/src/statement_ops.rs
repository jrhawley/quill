//! Multiple operations for working with `Statements`.

use chrono::{Datelike, Duration, Local, NaiveDate, Weekday};
use kronos::{Grain, Grains, Shim, TimeSequence};
use std::{path::Path, slice::Iter};

use crate::{IgnoredStatements, ObservedStatement, Statement, StatementStatus};

/// Calculate the next weekday from a given date
pub fn next_weekday_date(d: NaiveDate) -> NaiveDate {
    match d.weekday() {
        Weekday::Sat => Grains(Grain::Day)
            .future(&(d + Duration::days(2)).and_hms(0, 0, 0))
            .next()
            .unwrap()
            .start
            .date(),
        Weekday::Sun => Grains(Grain::Day)
            .future(&(d + Duration::days(1)).and_hms(0, 0, 0))
            .next()
            .unwrap()
            .start
            .date(),
        _ => d,
    }
}

/// Calculate the previous weekday from a given date
pub fn prev_weekday_date(d: NaiveDate) -> NaiveDate {
    match d.weekday() {
        Weekday::Sat => Grains(Grain::Day)
            .future(&(d - Duration::days(1)).and_hms(0, 0, 0))
            .next()
            .unwrap()
            .start
            .date(),
        Weekday::Sun => Grains(Grain::Day)
            .future(&(d - Duration::days(2)).and_hms(0, 0, 0))
            .next()
            .unwrap()
            .start
            .date(),
        _ => d,
    }
}

/// Calculate the next periodic date starting from a given date.
pub fn next_date_from_given<'a>(from: &NaiveDate, period: &Shim<'a>) -> NaiveDate {
    // need to shift date  by one day, because of how future is called
    let d = period
        .future(&(*from + Duration::days(1)).and_hms(0, 0, 0))
        .next()
        .unwrap()
        .start
        .date();
    // adjust for weekends
    // still adding days since statements are typically released after weekends, not before
    next_weekday_date(d)
}

/// Calculate the next periodic date starting from today.
pub fn next_date_from_today<'a>(period: &Shim<'a>) -> NaiveDate {
    let today = Local::now().naive_local().date();
    next_date_from_given(&today, period)
}

/// Calculate the most recent periodic date before a given date.
pub fn prev_date_from_given<'a>(from: &NaiveDate, period: &Shim<'a>) -> NaiveDate {
    // find the next statement
    let d = period
        .past(&from.and_hms(0, 0, 0))
        .next()
        .unwrap()
        .start
        .date();
    // adjust for weekends
    // still adding days since statements are typically released after weekends, not before
    prev_weekday_date(d)
}

/// Calculate the most recent periodic date before today
pub fn prev_date_from_today<'a>(period: &Shim<'a>) -> NaiveDate {
    let today = Local::now().naive_local().date();
    prev_date_from_given(&today, period)
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
    use kronos::step_by;
    use std::path::Path;

    use super::*;
    use crate::{IgnoredStatements, ObservedStatement, Statement, StatementStatus};

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[track_caller]
    fn check_next_weekday_date(input_date: NaiveDate, expected: NaiveDate) {
        let observed = next_weekday_date(input_date);

        assert_eq!(expected, observed);
    }

    #[test]
    fn all_next_weekday_date() {
        let wednesday = NaiveDate::from_ymd(2021, 12, 1);
        let thursday = NaiveDate::from_ymd(2021, 12, 2);
        let friday = NaiveDate::from_ymd(2021, 12, 3);
        let saturday = NaiveDate::from_ymd(2021, 12, 4);
        let sunday = NaiveDate::from_ymd(2021, 12, 5);
        let monday = NaiveDate::from_ymd(2021, 12, 6);
        let tuesday = NaiveDate::from_ymd(2021, 12, 7);

        check_next_weekday_date(wednesday, wednesday);
        check_next_weekday_date(thursday, thursday);
        check_next_weekday_date(friday, friday);
        check_next_weekday_date(saturday, monday);
        check_next_weekday_date(sunday, monday);
        check_next_weekday_date(monday, monday);
        check_next_weekday_date(tuesday, tuesday);
    }

    #[track_caller]
    fn check_prev_weekday_date(input_date: NaiveDate, expected: NaiveDate) {
        let observed = prev_weekday_date(input_date);

        assert_eq!(expected, observed);
    }

    #[test]
    fn all_prev_weekday_date() {
        let wednesday = NaiveDate::from_ymd(2021, 12, 1);
        let thursday = NaiveDate::from_ymd(2021, 12, 2);
        let friday = NaiveDate::from_ymd(2021, 12, 3);
        let saturday = NaiveDate::from_ymd(2021, 12, 4);
        let sunday = NaiveDate::from_ymd(2021, 12, 5);
        let monday = NaiveDate::from_ymd(2021, 12, 6);
        let tuesday = NaiveDate::from_ymd(2021, 12, 7);

        check_prev_weekday_date(wednesday, wednesday);
        check_prev_weekday_date(thursday, thursday);
        check_prev_weekday_date(friday, friday);
        check_prev_weekday_date(saturday, friday);
        check_prev_weekday_date(sunday, friday);
        check_prev_weekday_date(monday, monday);
        check_prev_weekday_date(tuesday, tuesday);
    }
    #[track_caller]
    fn check_next_date_from_given<'a>(
        input_date: NaiveDate,
        input_shim: &Shim<'a>,
        expected: NaiveDate,
    ) {
        let observed = next_date_from_given(&input_date, input_shim);

        assert_eq!(expected, observed);
    }

    #[test]
    fn all_next_date_from_given() {
        let wednesday = NaiveDate::from_ymd(2021, 12, 1);
        let thursday = NaiveDate::from_ymd(2021, 12, 2);
        let friday = NaiveDate::from_ymd(2021, 12, 3);
        let saturday = NaiveDate::from_ymd(2021, 12, 4);
        let sunday = NaiveDate::from_ymd(2021, 12, 5);
        let monday = NaiveDate::from_ymd(2021, 12, 6);
        let tuesday = NaiveDate::from_ymd(2021, 12, 7);
        let next_wednesday = NaiveDate::from_ymd(2021, 12, 8);

        // step every single day
        let next_day_shim = Shim::new(step_by(Grains(Grain::Day), 1));

        check_next_date_from_given(wednesday, &next_day_shim, thursday);
        check_next_date_from_given(thursday, &next_day_shim, friday);
        check_next_date_from_given(friday, &next_day_shim, monday);
        check_next_date_from_given(saturday, &next_day_shim, monday);
        check_next_date_from_given(sunday, &next_day_shim, monday);
        check_next_date_from_given(monday, &next_day_shim, tuesday);
        check_next_date_from_given(tuesday, &next_day_shim, next_wednesday);
    }

    #[track_caller]
    fn check_prev_date_from_given<'a>(
        input_date: NaiveDate,
        input_shim: &Shim<'a>,
        expected: NaiveDate,
    ) {
        let observed = prev_date_from_given(&input_date, input_shim);

        assert_eq!(expected, observed);
    }

    #[test]
    fn all_prev_date_from_given() {
        let wednesday = NaiveDate::from_ymd(2021, 12, 1);
        let thursday = NaiveDate::from_ymd(2021, 12, 2);
        let friday = NaiveDate::from_ymd(2021, 12, 3);
        let saturday = NaiveDate::from_ymd(2021, 12, 4);
        let sunday = NaiveDate::from_ymd(2021, 12, 5);
        let monday = NaiveDate::from_ymd(2021, 12, 6);
        let tuesday = NaiveDate::from_ymd(2021, 12, 7);
        let next_wednesday = NaiveDate::from_ymd(2021, 12, 8);

        // step every single day
        let next_day_shim = Shim::new(step_by(Grains(Grain::Day), 1));

        check_prev_date_from_given(thursday, &next_day_shim, wednesday);
        check_prev_date_from_given(friday, &next_day_shim, thursday);
        check_prev_date_from_given(saturday, &next_day_shim, friday);
        check_prev_date_from_given(sunday, &next_day_shim, friday);
        check_prev_date_from_given(monday, &next_day_shim, friday);
        check_prev_date_from_given(tuesday, &next_day_shim, monday);
        check_prev_date_from_given(next_wednesday, &next_day_shim, tuesday);
    }

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
    fn test_empty_dates() {
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
    fn test_missing() {
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
    fn test_available() {
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
    fn test_ignore() {
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
