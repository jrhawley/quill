//! Stepping dates backwards.

use chrono::{Datelike, Duration, Local, NaiveDate, Weekday};
use kronos::{Grain, Grains, Shim, TimeSequence};

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

#[cfg(test)]
mod tests {
    use super::*;
    use kronos::step_by;

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
}
