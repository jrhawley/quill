//! Information for a single account.

use chrono::prelude::*;
use chrono::Duration;
use kronos::TimeSequence;
use kronos::{step_by, Grain, Grains, LastOf, NthOf, Shim};
use log::warn;
use std::convert::TryFrom;
use std::fmt::{Debug, Display};
use std::io;
use std::path::Path;
use std::path::PathBuf;
use toml::Value;
use walkdir::WalkDir;

use crate::config::utils::expand_tilde;
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
        dir: &Path,
    ) -> Account<'a> {
        // print warning if the directory cannot be found
        if !dir.exists() {
            warn!("Account `{}` with directory `{}` cannot be found. Statements may not be processed properly.", name, dir.display());
        }
        Account {
            name: String::from(name),
            institution: String::from(institution),
            statement_first: first,
            statement_period: period,
            statement_fmt: String::from(fmt),
            dir: dir.to_path_buf(),
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
            // get the next date after the current iterated date
            iter_date = self.next_statement_date(iter_date);
        }
        stmnts.sort();
        return stmnts;
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

            // if current statement and previous date are equal, advance both iterators
            if curr_avail.date() == prev_req {
                pairs.push((prev_req, Some(curr_avail)));
                prev_req = curr_req;
                cr = req_it.next();
                ca = avail_it.next();
                is_prev_assigned = false;
            // if current statement is earlier than the current required one
            } else if curr_avail.date() < curr_req {
                // assign current statement to previous date if it hasn't been assigned yet
                // and when this statement is closer in date to the previous required date
                if !is_prev_assigned
                    && ((curr_avail.date() - prev_req) < (curr_req - curr_avail.date()))
                {
                    pairs.push((prev_req, Some(curr_avail)));
                    prev_req = curr_req;
                    cr = req_it.next();
                    ca = avail_it.next();
                    is_prev_assigned = false;
                // otherwise assign the previous statement as missing
                // and assign the current statement to the current required date
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
            // if current statement is the same date the required date match them
            } else if curr_avail.date() == curr_req {
                if !is_prev_assigned {
                    pairs.push((prev_req, None));
                }
                pairs.push((curr_req, Some(curr_avail)));
                prev_req = curr_req;
                cr = req_it.next();
                ca = avail_it.next();
                is_prev_assigned = true;
            // if current statement is in the future of the current required date
            // leave it for the future
            } else {
                if !is_prev_assigned {
                    pairs.push((prev_req, None));
                }
                prev_req = curr_req;
                cr = req_it.next();
                is_prev_assigned = false;
            }
        }

        // check that the previous required date is pushed properly
        // works regardless of whether ca is something or None
        if !is_prev_assigned {
            pairs.push((prev_req, ca));
        }
        // push out remaining pairs, as needed
        // if remaining required dates but no more available statements
        // don't need to check available statements if no more are required
        if cr.is_some() {
            // push remaining missing statement pairs
            while let Some(curr_req) = cr {
                pairs.push((curr_req, None));
                cr = req_it.next();
            }
        }
        Ok(pairs)
    }

    /// Identify all missing statements by comparing all possible and all downloaded statements
    pub fn missing_statements(&self) -> io::Result<Vec<Date>> {
        let pairs = self.match_statements()?;
        let missing: Vec<Date> = pairs
            .iter()
            .filter(|(_, stmt)| stmt.is_none())
            .map(|(d, _)| d.to_owned())
            .collect();
        Ok(missing)
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
    type Error = std::io::Error;
    fn try_from(props: &Value) -> Result<Self, Self::Error> {
        // extract name, if available
        let name = match props.get("name") {
            Some(Value::String(n)) => n.as_str(),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "No name for account",
                ))
            }
        };

        // extract and lookup corresponding institution
        let institution = match props.get("institution") {
            Some(Value::String(i)) => i.as_str(),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Account missing institution",
                ))
            }
        };

        // extract statement file name format
        let fmt = match props.get("statement_fmt") {
            Some(Value::String(f)) => f.as_str(),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "No statement name format for account",
                ))
            }
        };

        // extract directory containing statements
        let dir = match props.get("dir") {
            Some(Value::String(p)) => {
                // store the path
                let path = Path::new(p);
                // replace any tildes
                let non_tilded_path = expand_tilde(path).unwrap_or(path.to_path_buf());
                // make the path absolute
                match non_tilded_path.canonicalize() {
                    Ok(ap) => ap,
                    Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, e)),
                }
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "No directory account specified",
                ))
            }
        };

        // extract first statement date
        let first = match props.get("first_date") {
            Some(Value::Datetime(d)) => {
                match Date::parse_from_str(&d.to_string(), "%Y-%m-%dT%H:%M:%S%:z") {
                    Ok(d) => d,
                    Err(e) => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Error parsing statement date format",
                        ))
                    }
                }
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "No date for first statement",
                ))
            }
        };

        // extract statement period
        let period = match props.get("statement_period") {
            Some(Value::Array(p)) => {
                // check if using LastOf or Nth of to generate dates
                let mut is_lastof = false;
                if p.len() != 4 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Improperly formatted statement period",
                    ));
                }
                let nth: usize = match &p[0] {
                    Value::Integer(n) => {
                        if *n < 0 {
                            is_lastof = true;
                        }
                        (*n).abs() as usize
                    }
                    _ => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Non-integer for `nth` statement period",
                        ))
                    }
                };
                let mth: usize = match &p[3] {
                    Value::Integer(m) => *m as usize,
                    _ => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Non-integer for `mth` statement period",
                        ))
                    }
                };
                let x: Grains;
                let y: Grains;
                if let Value::String(x_str) = &p[1] {
                    x = match x_str.as_str() {
                        "Second" => Grains(Grain::Second),
                        "Minute" => Grains(Grain::Minute),
                        "Hour" => Grains(Grain::Hour),
                        "Day" => Grains(Grain::Day),
                        "Week" => Grains(Grain::Week),
                        "Month" => Grains(Grain::Month),
                        "Quarter" => Grains(Grain::Quarter),
                        "Half" => Grains(Grain::Half),
                        "Year" => Grains(Grain::Year),
                        "Lustrum" => Grains(Grain::Lustrum),
                        "Decade" => Grains(Grain::Decade),
                        "Century" => Grains(Grain::Century),
                        "Millennium" => Grains(Grain::Millenium),
                        _ => Grains(Grain::Day),
                    };
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Non-string for `x` statement period",
                    ));
                }
                if let Value::String(y_str) = &p[2] {
                    y = match y_str.as_str() {
                        "Second" => Grains(Grain::Second),
                        "Minute" => Grains(Grain::Minute),
                        "Hour" => Grains(Grain::Hour),
                        "Day" => Grains(Grain::Day),
                        "Week" => Grains(Grain::Week),
                        "Month" => Grains(Grain::Month),
                        "Quarter" => Grains(Grain::Quarter),
                        "Half" => Grains(Grain::Half),
                        "Year" => Grains(Grain::Year),
                        "Lustrum" => Grains(Grain::Lustrum),
                        "Decade" => Grains(Grain::Decade),
                        "Century" => Grains(Grain::Century),
                        "Millennium" => Grains(Grain::Millenium),
                        _ => Grains(Grain::Day),
                    };
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Non-string for `y` statement period",
                    ));
                }
                let y_step = step_by(y, mth);
                // return the TimeSequence object
                if is_lastof {
                    Shim::new(LastOf(nth, x, y_step))
                } else {
                    Shim::new(NthOf(nth, x, y_step))
                }
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Improperly formatted statement period",
                ))
            }
        };

        Ok(Account::new(
            name,
            institution,
            first,
            period,
            fmt,
            dir.as_path(),
        ))
    }
}
