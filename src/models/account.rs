use chrono::prelude::*;
use chrono::Duration;
use kronos::{Grain, Grains, Shim, TimeSequence};
use std::collections::HashMap;
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
    pub fn statement_dates(&self) -> Vec<Date> {
        let mut stmnts = Vec::new(); // statement Dates to be returned
        let now = Date(Local::today().naive_local());
        let mut iter_date = self.next_statement_date(self.statement_first);
        stmnts.push(self.statement_first);
        while iter_date <= now {
            stmnts.push(iter_date);
            iter_date = self.next_statement_date(iter_date);
        }
        return stmnts;
    }
    /// Check the daccount's irectory for all downloaded statements
    pub fn downloaded_statements(&self) -> HashMap<Date, PathBuf> {
        let false_date = Date::from_ymd(1900, 01, 01);
        // all statements in the directory
        let stmts: Vec<PathBuf> = read_dir(self.dir.as_path())
            .unwrap()
            .map(|p| p.unwrap().path())
            .collect();
        // dates from the statement names
        let dates: Vec<Statement> = stmts
            .iter()
            .map(|p| Statement::from_path(p, &self.statement_fmt))
            .collect();
        let mut hash: HashMap<Date, PathBuf> = HashMap::new();
        for (s, d) in stmts.into_iter().zip(dates.into_iter()) {
            if d.date() != false_date {
                hash.insert(d.date(), s);
            }
        }
        return hash;
    }
    /// Match expected and downloaded statements
    pub fn match_statements(&self) -> (Vec<Date>, Vec<Date>) {
        // get expected statements
        let mut required = self.statement_dates();
        required.sort();
        // get downloaded statements
        let mut available: Vec<Date> = self.downloaded_statements().keys().map(|&d| d).collect();
        available.sort();

        // find 1:1 mapping of required dates to downloaded dates
        // iterators over sorted dates
        let mut req_it = required.into_iter();
        let mut avail_it = available.into_iter();

        // placeholder for previous required statement
        // can guarantee the first statement exists
        let mut pr: Date = req_it.next().unwrap();
        // placeholders for results of iteration
        let mut cr = req_it.next();
        let mut ca = avail_it.next();
        return (required, available);
    }
    /// Identify all missing statements by comparing all possible and all downloaded statements
    pub fn missing_statements(&self) -> Vec<Date> {
        let mut required = self.statement_dates();
        required.sort();
        let mut available: Vec<Date> = self.downloaded_statements().keys().map(|&d| d).collect();
        available.sort();
        let mut missing: Vec<Date> = vec![];
        // find 1:1 mapping of required dates to downloaded dates
        // iterators over sorted dates
        let mut req_it = required.into_iter();
        let mut avail_it = available.into_iter();
        // placeholder previous required statement
        let mut prev_req: Date = req_it.next().unwrap();
        // placeholders for results of iteration
        let mut cr = req_it.next();
        let mut ca = avail_it.next();
        // if not at the end of one of the statement iterators
        while ca != None && cr != None {
            let curr_avail = ca.unwrap();
            let curr_req = cr.unwrap();
            // if the next available statement is not between these dates, prev_req is missing
            if curr_avail >= curr_req {
                missing.push(prev_req);
                // move to next iteration of required dates
                prev_req = curr_req;
                cr = req_it.next();
            } else {
                // move to next iteration of required dates
                prev_req = curr_req;
                cr = req_it.next();
                // and also advance to the next available date
                ca = avail_it.next();
            }
        }
        // if no more available statements
        if ca == None {
            missing.push(prev_req);
        }
        // push all remaining required statement dates to missing, if possible
        while cr != None {
            let curr_req = cr.unwrap();
            missing.push(curr_req);
            cr = req_it.next();
        }
        return missing;
    }
}

impl<'a> Display for Account<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.institution)
    }
}
