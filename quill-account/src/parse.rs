//! Utilities for converting to and from models and data types.

use chrono::NaiveDate;
use kronos::{step_by, Grain, Grains, LastOf, NthOf, Shim};
use std::{
    io::{Error, ErrorKind, Result},
    path::{Path, PathBuf},
    str::FromStr,
};
use toml::{value::Index, Value};

/// Generalized function to extract a string from a TOML value.
/// If the key is not found as a property, then return the provided error.
fn parse_str_from_toml<I>(key: I, props: &Value, err: Error) -> Result<&str>
where
    I: Index,
{
    match props.get(key) {
        Some(Value::String(s)) => Ok(s.as_str()),
        _ => return Err(err),
    }
}

/// Extract the account name from a TOML Value
pub(super) fn parse_account_name(props: &Value) -> Result<&str> {
    let err = Error::new(ErrorKind::NotFound, "No name for account");
    parse_str_from_toml("name", props, err)
}

/// Extract the account's institution from a TOML Value
pub(super) fn parse_institution_name(props: &Value) -> Result<&str> {
    let err = Error::new(ErrorKind::InvalidData, "Account missing institution");
    parse_str_from_toml("institution", props, err)
}

/// Extract the date format for a statement filename
pub(super) fn parse_statement_format(props: &Value) -> Result<&str> {
    let err = Error::new(
        ErrorKind::InvalidData,
        "No statement name format for account",
    );
    parse_str_from_toml("statement_fmt", props, err)
}

/// Extract the directory containing an account's statements
pub(super) fn parse_account_directory(props: &Value) -> Result<PathBuf> {
    let err = Error::new(ErrorKind::NotFound, "No directory account specified");
    match parse_str_from_toml("dir", props, err) {
        Ok(d) => {
            // store the path
            let path = Path::new(d);
            // replace any tildes
            let non_tilded_path = expand_tilde(path).unwrap_or(path.to_path_buf());
            // make the path absolute
            match non_tilded_path.canonicalize() {
                Ok(ap) => Ok(ap),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

/// Extract the date of the account's first statement
pub(super) fn parse_first_statement_date(props: &Value) -> Result<NaiveDate> {
    let err_not_found = Error::new(ErrorKind::NotFound, "No date for first statement");
    let err_parsing_date = Error::new(
        ErrorKind::InvalidData,
        "Error parsing statement date format",
    );

    match props.get("first_date") {
        Some(Value::Datetime(d)) => match NaiveDate::from_str(&d.to_string()) {
            Ok(d) => Ok(d),
            Err(_) => Err(err_parsing_date),
        },
        _ => Err(err_not_found),
    }
}

/// Extract the statement period for an account
pub(super) fn parse_statement_period<'a>(props: &Value) -> Result<Shim<'a>> {
    let err_invalid_fmt = Error::new(
        ErrorKind::InvalidData,
        "Improperly formatted statement period (4 values required)",
    );
    let err_n_non_int = Error::new(
        ErrorKind::InvalidData,
        "Non-integer for `nth` statement period",
    );
    let err_m_non_int = Error::new(
        ErrorKind::InvalidData,
        "Non-integer for `mth` statement period",
    );
    let err_x_non_str = Error::new(
        ErrorKind::InvalidData,
        "Non-string provided for `x` statement period",
    );
    let err_y_non_str = Error::new(
        ErrorKind::InvalidData,
        "Non-string provided for `y` statement period",
    );

    match props.get("statement_period") {
        Some(Value::Array(p)) => {
            // check if using LastOf or Nth of to generate dates
            let mut is_lastof = false;
            if p.len() != 4 {
                return Err(err_invalid_fmt);
            }
            let nth: usize = match &p[0] {
                Value::Integer(n) => {
                    if *n < 0 {
                        is_lastof = true;
                    }
                    (*n).abs() as usize
                }
                _ => return Err(err_n_non_int),
            };
            let mth: usize = match &p[3] {
                Value::Integer(m) => *m as usize,
                _ => return Err(err_m_non_int),
            };
            let x = value_to_grains(&p[1], err_x_non_str)?;
            let y = value_to_grains(&p[2], err_y_non_str)?;

            let y_step = step_by(y, mth);
            // return the TimeSequence object
            if is_lastof {
                Ok(Shim::new(LastOf(nth, x, y_step)))
            } else {
                Ok(Shim::new(NthOf(nth, x, y_step)))
            }
        }
        _ => Err(err_invalid_fmt),
    }
}

/// Convert a TOML Value to a Grains, if possible
fn value_to_grains(v: &Value, err: Error) -> Result<Grains> {
    match v {
        Value::String(s) => Ok(str_to_grains(s)),
        _ => Err(err),
    }
}

/// Convert a string to a Grains
fn str_to_grains(s: &str) -> Grains {
    match s {
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
        // this is a spelling mistake in the `kronos` library
        "Millennium" => Grains(Grain::Millenium),
        _ => Grains(Grain::Day),
    }
}
