//! Information for a single account.

use chrono::prelude::*;
use chrono::Duration;
use kronos::{Shim, TimeSequence};
use log::warn;
use serde::Serialize;
use std::convert::TryFrom;
use std::fmt::{Debug, Display};
use std::io;
use std::path::Path;
use std::path::PathBuf;
use toml::value::Datetime;
use toml::Value;
use walkdir::WalkDir;

use crate::models::Date;
use crate::models::Statement;

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
    pub fn downloaded_statements(&self) -> io::Result<Vec<Statement>> {
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
        stmts.sort_by(|a, b| a.date().partial_cmp(&b.date()).unwrap());

        Ok(stmts)
    }

    /// Match expected and downloaded statements
    pub fn match_statements(&self) -> io::Result<Vec<(Date, Option<Statement>)>> {
        // get expected statements
        let required = self.statement_dates();
        // get downloaded statements
        let available = self.downloaded_statements()?;
        pair_dates_statements(&required, &available)
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
    dates: &Vec<Date>,
    stmts: &Vec<Statement>,
) -> io::Result<Vec<(Date, Option<Statement>)>> {
    // iterators over sorted dates
    let mut req_it = dates.iter();
    let mut avail_it = stmts.iter();
    let mut pairs: Vec<(Date, Option<Statement>)> = vec![];

    // placeholder for previous required statement
    // if there is no first statement required
    // (i.e. first statement is in the future), then just return empty
    let mut prev_req = match req_it.next() {
        Some(d) => d,
        None => return Ok(vec![]),
    };
    // placeholders for results of iteration
    let mut cr = req_it.next();
    let mut ca = avail_it.next();

    // keep track of when `prev_req` has been properly paired
    let mut is_prev_assigned = false;
    while cr.is_some() && ca.is_some() {
        let curr_avail = ca.unwrap();
        let curr_req = cr.unwrap();

        // if current statement and previous date are equal, advance both iterators
        if curr_avail.date() == *prev_req {
            pairs.push((prev_req.clone(), Some(curr_avail.clone())));
            prev_req = curr_req;
            cr = req_it.next();
            ca = avail_it.next();
            is_prev_assigned = false;
        // if current statement is earlier than the current required one
        } else if curr_avail.date() < *curr_req {
            // assign current statement to previous date if it hasn't been assigned yet
            // and when this statement is closer in date to the previous required date
            if !is_prev_assigned
                && ((curr_avail.date() - *prev_req) < (*curr_req - curr_avail.date()))
            {
                pairs.push((prev_req.clone(), Some(curr_avail.clone())));
                prev_req = curr_req;
                cr = req_it.next();
                ca = avail_it.next();
                is_prev_assigned = false;
            // otherwise assign the previous statement as missing
            // and assign the current statement to the current required date
            } else {
                if !is_prev_assigned {
                    pairs.push((prev_req.clone(), None));
                }
                pairs.push((curr_req.clone(), Some(curr_avail.clone())));
                prev_req = curr_req;
                cr = req_it.next();
                ca = avail_it.next();
                is_prev_assigned = true;
            }
        // if current statement is the same date the required date match them
        } else if curr_avail.date() == *curr_req {
            if !is_prev_assigned {
                pairs.push((prev_req.clone(), None));
            }
            pairs.push((curr_req.clone(), Some(curr_avail.clone())));
            prev_req = curr_req;
            cr = req_it.next();
            ca = avail_it.next();
            is_prev_assigned = true;
        // if current statement is in the future of the current required date
        // leave it for the future
        } else {
            if !is_prev_assigned {
                pairs.push((prev_req.clone(), None));
            }
            prev_req = curr_req;
            cr = req_it.next();
            is_prev_assigned = false;
        }
    }

    // check that the previous required date is pushed properly
    // works regardless of whether ca is something or None
    if !is_prev_assigned {
        let ca_to_push = match ca {
            Some(s) => Some(s.clone()),
            None => None,
        };
        pairs.push((prev_req.clone(), ca_to_push));
    }
    // push out remaining pairs, as needed
    // if remaining required dates but no more available statements
    // don't need to check available statements if no more are required
    if cr.is_some() {
        // push remaining missing statement pairs
        while let Some(curr_req) = cr {
            pairs.push((curr_req.clone(), None));
            cr = req_it.next();
        }
    }

    Ok(pairs)
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
