use chrono::prelude::*;
use chrono::Duration;
use kronos::{Grain, Grains, Shim, TimeSequence};
use std::fmt::Display;
use std::fs::read_dir;
use std::path::PathBuf;

use crate::models::date::Date;
use crate::models::statement::Statement;

#[derive(Clone)]
/// Information related to an account, its billing period, and where to find the bills
pub struct Account<'a> {
    name: String,
    institution: String,
    statement_first: Date,
    statement_period: Shim<'a>,
    statement_fmt: String,
    dir: PathBuf,
}

impl<'a> Account<'a> {
    /// Declare a new Account
    pub fn new(
        name: &str,
        institution: &str,
        first: Date,
        period: Shim<'a>,
        fmt: &str,
        dir: PathBuf,
    ) -> Account<'a> {
        Account {
            name: String::from(name),
            institution: String::from(institution),
            statement_first: first,
            statement_period: period,
            statement_fmt: String::from(fmt),
            dir,
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

    /// Calculate the most recent statement before a given date for the account
    pub fn prev_statement_date(&self, date: Date) -> Date {
        // find the next statement
        let d = self
            .statement_period
            .past(&date.and_hms(0, 0, 0))
            .next()
            .unwrap()
            .start
            .date();
        // adjust for weekends
        // still adding days since statements are typically released after weekends, not before
        match d.weekday() {
            Weekday::Sat => Date(
                Grains(Grain::Day)
                    .future(&(d + Duration::days(2)).and_hms(0, 0, 0))
                    .next()
                    .unwrap()
                    .start
                    .date(),
            ),
            Weekday::Sun => Date(
                Grains(Grain::Day)
                    .future(&(d + Duration::days(1)).and_hms(0, 0, 0))
                    .next()
                    .unwrap()
                    .start
                    .date(),
            ),
            _ => Date(d),
        }
    }

    /// Print the most recent statement before today for the account
    pub fn prev_statement(&self) -> Date {
        self.prev_statement_date(Date(Local::now().naive_local().date()))
    }

    /// Calculate the next statement for the account from a given date
    pub fn next_statement_date(&self, date: Date) -> Date {
        // need to shift date  by one day, because of how future is called
        let d = self
            .statement_period
            .future(&(date.0 + Duration::days(1)).and_hms(0, 0, 0))
            .next()
            .unwrap()
            .start
            .date();
        match d.weekday() {
            Weekday::Sat => Date(
                Grains(Grain::Day)
                    .future(&(d + Duration::days(2)).and_hms(0, 0, 0))
                    .next()
                    .unwrap()
                    .start
                    .date(),
            ),
            Weekday::Sun => Date(
                Grains(Grain::Day)
                    .future(&(d + Duration::days(1)).and_hms(0, 0, 0))
                    .next()
                    .unwrap()
                    .start
                    .date(),
            ),
            _ => Date(d),
        }
    }
    /// Print the next statement for the account from today
    pub fn next_statement(&self) -> Date {
        self.next_statement_date(Date(Local::now().naive_local().date()))
    }

    /// List all statement dates for the account
    /// This list is guaranteed to be sorted, earliest first
    pub fn statement_dates(&self) -> Vec<Date> {
        let mut stmnts = Vec::new(); // statement Dates to be returned
        let now = Date(Local::today().naive_local());
        let mut iter_date = self.next_statement_date(self.statement_first);
        stmnts.push(self.statement_first);
        while iter_date <= now {
            stmnts.push(iter_date);
            iter_date = self.next_statement_date(iter_date);
        }
        stmnts.sort();
        return stmnts;
    }

    /// Check the account's directory for all downloaded statements
    /// This list is guaranteed to be sorted, earliest first
    pub fn downloaded_statements(&self) -> Vec<Statement> {
        // all statements in the directory
        let files: Vec<PathBuf> = read_dir(self.dir.as_path())
            .unwrap()
            .map(|p| p.unwrap().path())
            .filter(|p| p.is_file())
            .collect();
        // dates from the statement names
        let mut stmts: Vec<Statement> = files
            .iter()
            .map(|p| Statement::from_path(p, &self.statement_fmt))
            .collect();
        stmts.sort_by(|a, b| a.date().partial_cmp(&b.date()).unwrap());
        return stmts;
    }

    /// Match expected and downloaded statements
    pub fn match_statements(&self) -> Vec<(Date, Option<Statement>)> {
        // get expected statements
        let required = self.statement_dates();
        // get downloaded statements
        let available = self.downloaded_statements();

        // find 1:1 mapping of required dates to downloaded dates
        // iterators over sorted dates
        let mut req_it = required.into_iter();
        let mut avail_it = available.into_iter();
        let mut pairs: Vec<(Date, Option<Statement>)> = vec![];

        // placeholder for previous required statement
        // can guarantee the first required date exists
        let mut prev_req: Date = req_it.next().unwrap();
        // placeholders for results of iteration
        let mut cr = req_it.next();
        let mut ca = avail_it.next();

        // keep track of when `prev_req` has been properly paired
        let mut is_prev_assigned = false;
        while cr.is_some() && ca.is_some() {
            let curr_avail = ca.clone().unwrap();
            let curr_req = cr.unwrap();

            if curr_avail.date() == prev_req {
                // if current statement and previous date are equal, advance both iterators
                pairs.push((prev_req, Some(curr_avail)));
                prev_req = curr_req;
                cr = req_it.next();
                ca = avail_it.next();
                is_prev_assigned = false;
            } else if curr_avail.date() < curr_req {
                if !is_prev_assigned
                    && ((curr_avail.date() - prev_req) < (curr_req - curr_avail.date()))
                {
                    pairs.push((prev_req, Some(curr_avail)));
                    prev_req = curr_req;
                    cr = req_it.next();
                    ca = avail_it.next();
                    is_prev_assigned = false;
                } else {
                    if !is_prev_assigned {
                        pairs.push((prev_req, None));
                    }
                    pairs.push((curr_req, Some(curr_avail)));
                    prev_req = curr_req;
                    cr = req_it.next();
                    ca = avail_it.next();
                    is_prev_assigned = true;
                }
            } else if curr_avail.date() == curr_req {
                if !is_prev_assigned {
                    pairs.push((prev_req, None));
                }
                pairs.push((curr_req, Some(curr_avail)));
                prev_req = curr_req;
                cr = req_it.next();
                ca = avail_it.next();
                is_prev_assigned = true;
            } else {
                if !is_prev_assigned {
                    pairs.push((prev_req, None));
                }
                prev_req = curr_req;
                cr = req_it.next();
                is_prev_assigned = false;
            }
        }
        return pairs;
    }

    /// Identify all missing statements by comparing all possible and all downloaded statements
    pub fn missing_statements(&self) -> Vec<Date> {
        let pairs = self.match_statements();
        let missing: Vec<Date> = pairs
            .iter()
            .filter(|(_, stmt)| stmt.is_some())
            .map(|(d, _)| d.to_owned())
            .collect();
        return missing;
    }
}

impl<'a> Display for Account<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.institution)
    }
}
