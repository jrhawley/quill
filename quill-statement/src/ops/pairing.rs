//! Functions to pair dates with statements.

use crate::{
    next_date_from_given, IgnoredStatements, ObservedStatement, PairingError, Statement,
    StatementStatus,
};
use chrono::{Duration, Local, NaiveDate};
use kronos::Shim;
use std::slice::Iter;

/// A helper struct to navigate through the pairing operations
struct PairingIter<'a> {
    date_iter: Iter<'a, NaiveDate>,
    this_date: Option<&'a NaiveDate>,
    last_date: Option<&'a NaiveDate>,
    this_date_paired: bool,
    last_date_paired: bool,
    stmt_iter: Iter<'a, Statement>,
    this_stmt: Option<&'a Statement>,
    last_stmt: Option<&'a Statement>,
    this_stmt_paired: bool,
    last_stmt_paired: bool,
    ignore_iter: Iter<'a, NaiveDate>,
    this_ig: Option<&'a NaiveDate>,
    last_ig: Option<&'a NaiveDate>,
    pairs: Vec<ObservedStatement>,
}

impl<'a> PairingIter<'a> {
    /// Create a new iterator
    pub fn new(
        dates: &'a [NaiveDate],
        stmts: &'a [Statement],
        ignored: &'a IgnoredStatements,
    ) -> Self {
        let mut date_iter = dates.iter();
        let this_date = date_iter.next();

        let mut stmt_iter = stmts.iter();
        let this_stmt = stmt_iter.next();

        let mut ignore_iter = ignored.iter();
        let this_ig = ignore_iter.next();

        PairingIter {
            date_iter,
            this_date,
            last_date: None,
            this_date_paired: false,
            last_date_paired: false,
            stmt_iter,
            this_stmt,
            last_stmt: None,
            this_stmt_paired: false,
            last_stmt_paired: false,
            ignore_iter,
            this_ig,
            last_ig: None,
            pairs: vec![],
        }
    }

    /// Retrive the active date
    fn date(&self) -> Option<&NaiveDate> {
        self.this_date
    }

    /// Retrive the previous date
    fn previous_date(&self) -> Option<&NaiveDate> {
        self.last_date
    }

    /// Retrieve the active statement
    fn statement(&self) -> Option<&Statement> {
        self.this_stmt
    }
    /// Retrieve the active statement
    fn previous_statement(&self) -> Option<&Statement> {
        self.last_stmt
    }

    /// Retrieve the active statement's date
    fn statement_date(&self) -> Option<&NaiveDate> {
        match self.statement() {
            Some(stmt) => Some(stmt.date()),
            None => None,
        }
    }

    /// Retrieve the active ignored date
    fn ignore(&self) -> Option<&NaiveDate> {
        self.this_ig
    }

    /// Retrieve the pairings of dates and statements
    fn pairings(&self) -> &Vec<ObservedStatement> {
        &self.pairs
    }

    /// Move to the next date
    fn next_date(&mut self) {
        self.last_date = self.this_date;
        self.this_date = self.date_iter.next();
        self.last_date_paired = self.this_date_paired;
        self.this_date_paired = false;
    }

    /// Move to the next statement
    fn next_statement(&mut self) {
        self.last_stmt = self.this_stmt;
        self.this_stmt = self.stmt_iter.next();
        self.last_stmt_paired = self.this_stmt_paired;
        self.this_stmt_paired = false;
    }

    /// Move to the next statement
    fn next_ignore(&mut self) {
        self.last_ig = self.this_ig;
        self.this_ig = self.ignore_iter.next();
    }

    /// Push a new statement and status
    fn push_statement(&mut self, status: StatementStatus) -> Result<(), PairingError> {
        let this_stmt = match (self.date(), self.statement()) {
            (Some(date), Some(stmt)) => Statement::new(stmt.path(), date),
            (Some(date), None) => Statement::from(date),
            (None, _) => return Err(PairingError::NoneDateForPairing),
        };
        let obs_stmt = ObservedStatement::new(&this_stmt, status);

        self.pairs.push(obs_stmt);
        self.this_date_paired = true;
        self.next_date();

        Ok(())
    }

    /// Push a the previous statement and given status
    fn push_previous_statement(&mut self, status: StatementStatus) -> Result<(), PairingError> {
        let this_stmt = match (self.date(), self.previous_statement()) {
            (Some(date), Some(stmt)) => Statement::new(stmt.path(), date),
            (Some(date), None) => Statement::from(date),
            (None, _) => return Err(PairingError::NoneDateForPairing),
        };
        let obs_stmt = ObservedStatement::new(&this_stmt, status);

        self.pairs.push(obs_stmt);
        self.this_date_paired = true;
        self.next_date();

        Ok(())
    }

    /// Push a new statement and status
    fn push_date(&mut self, status: StatementStatus) -> Result<(), PairingError> {
        let this_stmt = match self.date() {
            Some(d) => Statement::from(d),
            None => return Err(PairingError::NoneDateForPairing),
        };
        let obs_stmt = ObservedStatement::new(&this_stmt, status);
        self.pairs.push(obs_stmt);
        self.next_date();

        Ok(())
    }

    /// Determine if the current statement's date is close enough to the current date
    fn statement_in_proximity(&self, stmt: Option<&Statement>) -> bool {
        let limit = Duration::days(3);

        if let (Some(d), Some(s)) = (self.date(), stmt) {
            if s.date() > d {
                *s.date() - *d < limit
            } else {
                *d - *s.date() < limit
            }
        } else {
            false
        }
    }

    /// Determine if the current statement is closer to the date than the previous statement
    fn this_statement_is_closest(&self) -> bool {
        match (self.date(), self.statement(), self.previous_statement()) {
            (Some(date), Some(this_stmt), Some(last_stmt)) => {
                let this_diff = match this_stmt.date() > date {
                    true => *this_stmt.date() - *date,
                    false => *date - *this_stmt.date(),
                };
                let last_diff = match last_stmt.date() > date {
                    true => *last_stmt.date() - *date,
                    false => *date - *last_stmt.date(),
                };

                this_diff < last_diff
            }
            // this_stmt can't be closest if it doesn't exist
            (Some(_), None, Some(_)) => false,
            // this_stmt can't be further than None
            (Some(_), Some(_), None) => true,
            (_, _, _) => true,
        }
    }
}

/// Match elements of Dates and Statements together to find closest pairing.
/// Finds a 1:1 mapping of dates to statements, if possible.
pub fn pair_dates_statements(
    dates: &[NaiveDate],
    stmts: &[Statement],
    ignored: &IgnoredStatements,
) -> Result<Vec<ObservedStatement>, PairingError> {
    // iterators over sorted dates
    let mut pairs = PairingIter::new(dates, stmts, ignored);

    while pairs.date().is_some() {
        // fast forward the ignores
        while let (Some(ig_date), Some(date)) = (pairs.ignore(), pairs.date()) {
            if ig_date < date {
                pairs.next_ignore();
            } else {
                break;
            }
        }

        // check if the current date should be ignored
        if pairs.ignore() == pairs.date() {
            pairs.push_date(StatementStatus::Ignored)?;
            continue;
        }

        // fast forward the statements
        while let (Some(stmt), Some(date)) = (pairs.statement(), pairs.date()) {
            if stmt.date() < date {
                pairs.next_statement();
            } else {
                break;
            }
        }

        // check if the previous or current statement should be paired with the current date
        if pairs.statement_date() == pairs.date() {
            pairs.push_statement(StatementStatus::Available)?;
        } else if pairs.statement_in_proximity(pairs.statement())
            && pairs.this_statement_is_closest()
        {
            pairs.push_statement(StatementStatus::Available)?;
        } else if pairs.statement_in_proximity(pairs.previous_statement())
            && !pairs.this_statement_is_closest()
        {
            pairs.push_previous_statement(StatementStatus::Available)?;
        } else {
            // no other options means its missing
            pairs.push_date(StatementStatus::Missing)?;
        }
    }

    Ok(pairs.pairings().to_vec())
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
        let observed = pair_dates_statements(input_dates, input_stmts, input_ignored).unwrap();
        assert_eq!(expected, observed);
    }

    // A helper function for quickly created statments with a certain date
    fn blank_statement(year: i32, month: u32, day: u32) -> Statement {
        Statement::from(&NaiveDate::from_ymd(year, month, day))
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
        let input_stmts = &[blank_statement(2021, 9, 22), blank_statement(2021, 11, 22)];
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
    fn missing_ignored_available() {
        let input_dates = &[
            NaiveDate::from_ymd(2021, 9, 22),
            NaiveDate::from_ymd(2021, 10, 22),
            NaiveDate::from_ymd(2021, 11, 22),
        ];
        let input_stmts = &[blank_statement(2021, 9, 22)];
        let input_ignored = &IgnoredStatements::from(vec![NaiveDate::from_ymd(2021, 10, 22)]);

        let expected = vec![
            ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Available),
            ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Ignored),
            ObservedStatement::new(&blank_statement(2021, 11, 22), StatementStatus::Missing),
        ];

        check_pair_dates_statements(input_dates, input_stmts, input_ignored, expected);
    }

    /// When an ignored date doesn't perfectly line up with a statement date,
    /// it should be as if the date isn't being ignored.
    /// Trying when the ignored date is before the missing statement.
    #[test]
    fn mismatching_ignore_before_stmt() {
        let input_dates = &[
            NaiveDate::from_ymd(2021, 4, 5),
            NaiveDate::from_ymd(2021, 5, 3),
        ];
        let input_stmts = &[];
        let input_ignored = &IgnoredStatements::from(vec![NaiveDate::from_ymd(2021, 4, 1)]);

        let expected = vec![
            ObservedStatement::new(&blank_statement(2021, 4, 5), StatementStatus::Missing),
            ObservedStatement::new(&blank_statement(2021, 5, 3), StatementStatus::Missing),
        ];

        check_pair_dates_statements(input_dates, input_stmts, input_ignored, expected);
    }

    /// When an ignored date doesn't perfectly line up with a statement date,
    /// it should be as if the date isn't being ignored.
    /// Trying when the ignored date is after the missing statement.
    #[test]
    fn mismatching_ignore_between_missing_stmts() {
        let input_dates = &[
            NaiveDate::from_ymd(2021, 4, 5),
            NaiveDate::from_ymd(2021, 5, 3),
        ];
        let input_stmts = &[];
        let input_ignored = &IgnoredStatements::from(vec![NaiveDate::from_ymd(2021, 4, 6)]);

        let expected = vec![
            ObservedStatement::new(&blank_statement(2021, 4, 5), StatementStatus::Missing),
            ObservedStatement::new(&blank_statement(2021, 5, 3), StatementStatus::Missing),
        ];

        check_pair_dates_statements(input_dates, input_stmts, input_ignored, expected);
    }

    /// When an ignored date doesn't perfectly line up with a statement date,
    /// it should be as if the date isn't being ignored.
    /// Trying when the ignored date is before the available statement.
    #[test]
    fn mismatching_ignore_before_avail_stmts() {
        let input_dates = &[
            NaiveDate::from_ymd(2021, 4, 5),
            NaiveDate::from_ymd(2021, 5, 3),
        ];
        let input_stmts = &[blank_statement(2021, 4, 5), blank_statement(2021, 5, 3)];
        let input_ignored = &IgnoredStatements::from(vec![NaiveDate::from_ymd(2021, 4, 4)]);

        let expected = vec![
            ObservedStatement::new(&blank_statement(2021, 4, 5), StatementStatus::Available),
            ObservedStatement::new(&blank_statement(2021, 5, 3), StatementStatus::Available),
        ];

        check_pair_dates_statements(input_dates, input_stmts, input_ignored, expected);
    }

    /// When an ignored date doesn't perfectly line up with a statement date,
    /// it should be as if the date isn't being ignored.
    /// Trying when the ignored date is after the statement.
    #[test]
    fn mismatching_ignore_between_avail_stmts() {
        let input_dates = &[
            NaiveDate::from_ymd(2021, 4, 5),
            NaiveDate::from_ymd(2021, 5, 3),
        ];
        let input_stmts = &[blank_statement(2021, 4, 5), blank_statement(2021, 5, 3)];
        let input_ignored = &IgnoredStatements::from(vec![NaiveDate::from_ymd(2021, 4, 6)]);

        let expected = vec![
            ObservedStatement::new(&blank_statement(2021, 4, 5), StatementStatus::Available),
            ObservedStatement::new(&blank_statement(2021, 5, 3), StatementStatus::Available),
        ];

        check_pair_dates_statements(input_dates, input_stmts, input_ignored, expected);
    }

    /// When an ignored date doesn't perfectly line up with a statement date,
    /// it should be as if the date isn't being ignored.
    /// This shouldn't affect any future ignores that do line up
    #[test]
    fn independent_ignores() {
        let input_dates = &[
            NaiveDate::from_ymd(2021, 4, 5),
            NaiveDate::from_ymd(2021, 5, 3),
        ];
        let input_stmts = &[blank_statement(2021, 4, 5), blank_statement(2021, 5, 3)];
        let input_ignored = &IgnoredStatements::from(vec![
            NaiveDate::from_ymd(2021, 4, 6),
            NaiveDate::from_ymd(2021, 5, 3),
        ]);

        let expected = vec![
            ObservedStatement::new(&blank_statement(2021, 4, 5), StatementStatus::Available),
            ObservedStatement::new(&blank_statement(2021, 5, 3), StatementStatus::Ignored),
        ];

        check_pair_dates_statements(input_dates, input_stmts, input_ignored, expected);
    }

    /// When a statement date doesn't exactly line up with an expected date,
    /// it should still match.
    /// Check that a statement between two dates matches to the closest one in the past.
    #[test]
    fn stmt_mismatch_paired_with_closest_past() {
        let input_dates = &[
            NaiveDate::from_ymd(2021, 9, 22),
            NaiveDate::from_ymd(2021, 10, 22),
        ];
        let input_stmts = &[blank_statement(2021, 9, 23)];
        let input_ignored = &IgnoredStatements::empty();

        let expected = vec![
            ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Available),
            ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Missing),
        ];

        check_pair_dates_statements(input_dates, input_stmts, input_ignored, expected);
    }

    #[test]
    fn stmt_mismatch_paired_with_closest_future() {
        let input_dates = &[
            NaiveDate::from_ymd(2021, 9, 22),
            NaiveDate::from_ymd(2021, 10, 22),
        ];
        let input_stmts = &[blank_statement(2021, 9, 21)];
        let input_ignored = &IgnoredStatements::empty();

        let expected = vec![
            ObservedStatement::new(&blank_statement(2021, 9, 22), StatementStatus::Available),
            ObservedStatement::new(&blank_statement(2021, 10, 22), StatementStatus::Missing),
        ];

        check_pair_dates_statements(input_dates, input_stmts, input_ignored, expected);
    }
}
