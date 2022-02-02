//! Multiple operations for working with `Statements`.

use chrono::{Datelike, Duration, NaiveDate, Weekday};
use kronos::{Grain, Grains, TimeSequence};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
