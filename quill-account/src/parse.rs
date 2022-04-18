//! Utilities for converting to and from models and data types.

use chrono::NaiveDate;
use dirs::home_dir;
use kronos::{step_by, Grain, Grains, LastOf, NthOf, Shim};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};
use toml::{value::Index, Value};

use crate::AccountCreationError;

/// Replace the `~` character in any path with the home directory.
/// See <https://stackoverflow.com/a/54306906/7416009>
pub fn expand_tilde<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
    let p = path.as_ref();
    if !p.starts_with("~") {
        return Some(p.to_path_buf());
    }
    if p == Path::new("~") {
        return home_dir();
    }
    home_dir().map(|mut h| {
        if h == Path::new("/") {
            // base case: `h` root directory;
            // don't prepend extra `/`, just drop the tilde.
            p.strip_prefix("~").unwrap().to_path_buf()
        } else {
            h.push(p.strip_prefix("~/").unwrap());
            h
        }
    })
}

/// Generalized function to extract a string from a TOML value.
/// If the key is not found as a property, then return the provided error.
fn parse_str_from_toml<I>(
    key: I,
    props: &Value,
    err: AccountCreationError,
) -> Result<&str, AccountCreationError>
where
    I: Index,
{
    match props.get(key) {
        Some(Value::String(s)) => Ok(s.as_str()),
        _ => Err(err),
    }
}

/// Extract the account name from a TOML Value
pub(super) fn parse_account_name(props: &Value) -> Result<&str, AccountCreationError> {
    parse_str_from_toml("name", props, AccountCreationError::MissingAccountName)
}

/// Extract the account's institution from a TOML Value
pub(super) fn parse_institution_name(props: &Value) -> Result<&str, AccountCreationError> {
    parse_str_from_toml(
        "institution",
        props,
        AccountCreationError::MissingInstitutionName,
    )
}

/// Extract the date format for a statement filename
pub(super) fn parse_statement_format(props: &Value) -> Result<&str, AccountCreationError> {
    parse_str_from_toml(
        "statement_fmt",
        props,
        AccountCreationError::MissingStatementFormat,
    )
}

/// Extract the directory containing an account's statements
pub(super) fn parse_account_directory(props: &Value) -> Result<PathBuf, AccountCreationError> {
    match parse_str_from_toml(
        "dir",
        props,
        AccountCreationError::MissingStatementDirectory,
    ) {
        Ok(d) => {
            // store the path
            let path = Path::new(d);
            // replace any tildes
            let non_tilded_path = expand_tilde(path).unwrap_or_else(|| path.to_path_buf());
            // make the path absolute
            match non_tilded_path.canonicalize() {
                Ok(ap) => match ap.exists() {
                    true => Ok(ap),
                    false => Err(AccountCreationError::StatementDirectoryNotFound(ap)),
                },
                Err(_) => Err(AccountCreationError::StatementDirectoryNonCanonical(
                    path.to_path_buf(),
                )),
            }
        }
        Err(e) => Err(e),
    }
}

/// Extract the date of the account's first statement
pub(super) fn parse_first_statement_date(props: &Value) -> Result<NaiveDate, AccountCreationError> {
    match props.get("first_date") {
        Some(Value::Datetime(d)) => match NaiveDate::from_str(&d.to_string()) {
            Ok(d) => Ok(d),
            Err(_) => Err(AccountCreationError::InvalidFirstDate(d.to_string())),
        },
        _ => Err(AccountCreationError::MissingFirstDate),
    }
}

/// Extract the statement period for an account
pub(super) fn parse_statement_period<'a>(props: &Value) -> Result<Shim<'a>, AccountCreationError> {
    match props.get("statement_period") {
        Some(Value::Array(arr)) => parse_period_array(arr),
        _ => Err(AccountCreationError::MissingPeriod),
    }
}

/// Convert a TOML Value to a Grains, if possible
fn value_to_grains(v: &Value) -> Result<Grains, AccountCreationError> {
    match v {
        Value::String(s) => str_to_grains(s),
        _ => Err(AccountCreationError::InvalidPeriodGrainNotAString(
            v.as_str().unwrap_or("").to_string(),
        )),
    }
}

/// Convert a string to a Grains
fn str_to_grains(s: &str) -> Result<Grains, AccountCreationError> {
    match s {
        "Day" => Ok(Grains(Grain::Day)),
        "Week" => Ok(Grains(Grain::Week)),
        "Month" => Ok(Grains(Grain::Month)),
        "Quarter" => Ok(Grains(Grain::Quarter)),
        "Half" => Ok(Grains(Grain::Half)),
        "Year" => Ok(Grains(Grain::Year)),
        "Lustrum" => Ok(Grains(Grain::Lustrum)),
        "Decade" => Ok(Grains(Grain::Decade)),
        "Century" => Ok(Grains(Grain::Century)),
        // this is a spelling mistake in the `kronos` library
        "Millennium" | "Millenium" => Ok(Grains(Grain::Millenium)),
        _ => Err(AccountCreationError::InvalidPeriodGrainString(
            s.to_string(),
        )),
    }
}

/// Parse the entire array used to determine the statement period
fn parse_period_array<'a>(v: &Vec<Value>) -> Result<Shim<'a>, AccountCreationError> {
    if v.len() != 4 {
        return Err(AccountCreationError::InvalidPeriodIncorrectLength(v.len()));
    }

    let x = value_to_grains(&v[1])?;
    let mth = parse_mth_value(&v[2])?;
    let y = value_to_grains(&v[3])?;

    // return the TimeSequence object
    match &v[0] {
        // Value::Array(arr) => parse_multiple_periods(arr, x, mth, y),
        Value::Integer(nth) => Ok(parse_single_period(nth, &x, &mth, &y)),
        _ => Err(AccountCreationError::InvalidPeriodNonIntN),
    }
}

/// Turn a single set of period parameters into a `TimeSequence`
fn parse_single_period<'a>(n: &i64, x: &Grains, mth: &usize, y: &Grains) -> Shim<'a> {
    let (nth, is_lastof) = parse_nth_value(n);
    if is_lastof {
        Shim::new(LastOf(nth, x.clone(), step_by(y.clone(), *mth)))
    } else {
        Shim::new(NthOf(nth, x.clone(), step_by(y.clone(), *mth)))
    }
}

/// Parse the value stored as the `m`-th period input
fn parse_mth_value(v: &Value) -> Result<usize, AccountCreationError> {
    match v {
        Value::Integer(m) => Ok(*m as usize),
        _ => Err(AccountCreationError::InvalidPeriodNonIntM),
    }
}

/// Parse the value stored as the `n`-th period input
fn parse_nth_value(n: &i64) -> (usize, bool) {
    let val = (*n).abs() as usize;
    if *n < 0 {
        (val, true)
    } else {
        (val, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml::Value;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(4, result);
    }

    #[test]
    fn check_parse_mth_value_good() {
        let input = Value::Integer(2i64);
        let observed = parse_mth_value(&input);
        let expected = Ok(2usize);

        assert_eq!(expected, observed);
    }

    #[test]
    fn check_parse_mth_value_bad() {
        let input = Value::String("hello".to_string());
        let observed = parse_mth_value(&input);
        let expected = Err(AccountCreationError::InvalidPeriodNonIntM);

        assert_eq!(expected, observed);
    }

    #[test]
    fn check_parse_nth_value_negative() {
        let input: i64 = -1;
        let observed = parse_nth_value(&input);
        let expected = (1, true);

        assert_eq!(expected, observed);
    }

    #[test]
    fn check_parse_nth_value_positive() {
        let input: i64 = 2;
        let observed = parse_nth_value(&input);
        let expected = (2, false);

        assert_eq!(expected, observed);
    }
}
