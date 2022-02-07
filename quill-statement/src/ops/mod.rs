//! Multiple operations for working with `Statements`.

pub mod next_date;
pub mod pairing;
pub mod prev_date;

pub use next_date::{next_date_from_given, next_date_from_today, next_weekday_date};
pub use pairing::{expected_statement_dates, pair_dates_statements};
pub use prev_date::{prev_date_from_given, prev_date_from_today};
