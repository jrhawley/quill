use serde::{Serialize, Deserialize};
use chrono::prelude::*;
use chrono::{Datelike, IsoWeek, ParseResult};
use std::fmt::{Debug, Display};

pub type DateTime = chrono::NaiveDateTime;

#[derive(Clone, Copy, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
// a wrapper struct for the default NaiveDate struct for an alternative Display trait
pub struct Date(pub chrono::NaiveDate);

impl Date {
    pub fn and_hms(&self, hour: u32, min: u32, sec: u32) -> DateTime {
        self.0.and_hms(hour, min, sec)
    }
    pub fn from_ymd(year: i32, month: u32, day: u32) -> Date {
        Date(NaiveDate::from_ymd(year, month, day))
    }
    pub fn parse_from_str(s: &str, fmt: &str) -> ParseResult<Date> {
        match NaiveDate::parse_from_str(s, fmt) {
            Ok(d) => Ok(Date(d)),
            Err(e) => Err(e),
        }
    }
}

impl Datelike for Date {
    fn year(&self) -> i32 {
        self.0.year()
    }
    fn year_ce(&self) -> (bool, u32) {
        self.0.year_ce()
    }
    fn month(&self) -> u32 {
        self.0.month()
    }
    fn month0(&self) -> u32 {
        self.0.month0()
    }
    fn day(&self) -> u32 {
        self.0.day()
    }
    fn day0(&self) -> u32 {
        self.0.day0()
    }
    fn ordinal(&self) -> u32 {
        self.0.ordinal()
    }
    fn ordinal0(&self) -> u32 {
        self.0.ordinal0()
    }
    fn weekday(&self) -> Weekday {
        self.0.weekday()
    }
    fn iso_week(&self) -> IsoWeek {
        self.0.iso_week()
    }
    fn with_year(&self, year: i32) -> Option<Self> {
        match self.0.with_year(year) {
            Some(d) => Some(Date(d)),
            _ => None
        }
    }
    fn with_month(&self, month: u32) -> Option<Self> {
        match self.0.with_month(month) {
            Some(d) => Some(Date(d)),
            _ => None
        }
    }
    fn with_month0(&self, month0: u32) -> Option<Self> {
        match self.0.with_month0(month0) {
            Some(d) => Some(Date(d)),
            _ => None
        }
    }
    fn with_day(&self, day: u32) -> Option<Self> {
        match self.0.with_day(day) {
            Some(d) => Some(Date(d)),
            _ => None
        }
    }
    fn with_day0(&self, day0: u32) -> Option<Self> {
        match self.0.with_day0(day0) {
            Some(d) => Some(Date(d)),
            _ => None
        }
    }
    fn with_ordinal(&self, ordinal: u32) -> Option<Self> {
        match self.0.with_ordinal(ordinal) {
            Some(d) => Some(Date(d)),
            _ => None
        }
    }
    fn with_ordinal0(&self, ordinal0: u32) -> Option<Self> {
        match self.0.with_ordinal0(ordinal0) {
            Some(d) => Some(Date(d)),
            _ => None
        }
    }
    fn num_days_from_ce(&self) -> i32 {
        self.0.num_days_from_ce()
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.format("%Y-%m-%d"))
    }
}

impl Debug for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.format("%Y-%m-%d"))
    }
}
