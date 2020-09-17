use std::io::prelude::*;
use std::fs::File;
use std::fmt::Display;
use std::path::Path;
use std::collections::HashMap;
use toml::Value;
use toml::map::Map;
use kronos::{NthOf, Grains, Grain, step_by, Shim};

use crate::models::account::Account;
use crate::models::institution::Institution;
use crate::models::date::Date;

pub struct Config<'a> {
    institutions: HashMap<String, Institution>,
    accounts: HashMap<String, Account<'a>>,
}

impl<'a> Config<'a> {
    /// Get the list of institutions in the configuration
    pub fn institutions(&self) -> &HashMap<String, Institution> {
        return &self.institutions;
    }
    
    /// Get the list of accounts in the configuration
    pub fn accounts(&self) -> &HashMap<String, Account<'a>> {
        return &self.accounts;
    }
    
    /// Add a new account to the configuration
    pub fn add_account(&mut self, key: &str, props: &toml::Value) {
        // extract name, if available
        let name = match props.get("name") {
            Some(Value::String(n)) => n.to_string(),
            _ => panic!("No name for account"),
        };
        
        // extract and lookup corresponding institution
        let inst = match props.get("institution") {
            Some(Value::String(i)) => {
                // look up institution `i` in `conf` and return its reference
                self.institutions().get(i).unwrap().to_string()
            },
            _ => panic!("No appropriate name for institution"),
        };

        // extract statement file name format
        let fmt = match props.get("statement_fmt") {
            Some(Value::String(f)) => f.to_string(),
            _ => panic!("No statement name format for account"),
        };

        // extract directory containing statements
        let dir = match props.get("dir") {
            Some(Value::String(p)) => Path::new(p),
            _ => panic!("No directory for account"),
        };

        // extract first statement date
        let stmt_first = match props.get("first_date") {
            Some(Value::Datetime(d)) => {
                Date::parse_from_str(&d.to_string(), "%Y-%m-%dT%H:%M:%S%:z").unwrap()
            },
            _ => panic!("No date for first statement"),
        };

        // extract statement period
        let period = match props.get("statement_period") {
            Some(Value::Array(p)) => {
                if p.len() != 4 {
                    panic!("Improperly formatted statement period");
                }
                let nth: usize = match &p[0] {
                    Value::Integer(n) => *n as usize,
                    _ => panic!("Non-integer for `nth` statement period")
                };
                let mth: usize = match &p[3] {
                    Value::Integer(m) => *m as usize,
                    _ => panic!{"Non-integer for `mth` statement period"}
                };
                let x: Grains;
                let y: Grains;
                if let Value::String(x_str) = &p[1] {
                    x = match x_str.as_str() {
                        "Second"    => Grains(Grain::Second),
                        "Minute"    => Grains(Grain::Minute),
                        "Hour"      => Grains(Grain::Hour),
                        "Day"       => Grains(Grain::Day),
                        "Week"      => Grains(Grain::Week),
                        "Month"     => Grains(Grain::Month),
                        "Quarter"   => Grains(Grain::Quarter),
                        "Half"      => Grains(Grain::Half),
                        "Year"      => Grains(Grain::Year),
                        "Lustrum"   => Grains(Grain::Lustrum),
                        "Decade"    => Grains(Grain::Decade),
                        "Century"   => Grains(Grain::Century),
                        "Millenium" => Grains(Grain::Millenium),
                        _           => Grains(Grain::Day),
                    };
                } else {
                    panic!("Non-string for `x` statement period");
                }
                if let Value::String(y_str) = &p[2] {
                    y = match y_str.as_str() {
                        "Second"    => Grains(Grain::Second),
                        "Minute"    => Grains(Grain::Minute),
                        "Hour"      => Grains(Grain::Hour),
                        "Day"       => Grains(Grain::Day),
                        "Week"      => Grains(Grain::Week),
                        "Month"     => Grains(Grain::Month),
                        "Quarter"   => Grains(Grain::Quarter),
                        "Half"      => Grains(Grain::Half),
                        "Year"      => Grains(Grain::Year),
                        "Lustrum"   => Grains(Grain::Lustrum),
                        "Decade"    => Grains(Grain::Decade),
                        "Century"   => Grains(Grain::Century),
                        "Millenium" => Grains(Grain::Millenium),
                        _           => Grains(Grain::Day),
                    };
                } else {
                    panic!("Non-string for `y` statement period");
                }
                let y_step = step_by(y, mth);
                // return the NthOf object
                Shim::new(NthOf(nth, x, y_step))
            },
            _ => panic!("Improperly formatted statement period"),
        };
        // create account and push to conf
        // can't use serialization here for the entire account
        // because we have a more complex relationship between the Account struct and its components
        let a = Account {
            name: name,
            institution: inst,
            statement_first: stmt_first,
            statement_period: period,
            statement_fmt: fmt,
            dir: dir.to_path_buf(),
        };
        self.accounts.insert(key.to_string(), a);
        
    }
    
    /// Add a new institution to the configuration
    pub fn add_institution(&mut self, key: &str, props: &toml::Value) {
        // extract name, if available
        if let Some(Value::String(_)) = props.get("name") {
            // create institutions and push to conf using serialization
            let i: Institution = toml::from_str(&props.to_string()).unwrap();
            self.institutions.insert(key.to_string(), i);
        }
    }
}

impl<'a> Display for Config<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Institutions: {:?}",
            self.institutions.values()
        )
    }
}

/// Parse a TOML table for institutions and create Institution structs
fn parse_institutions<'a>(institutions: &Map<String, Value>, conf: &mut Config<'a>) {
    for (inst, props) in institutions {
        conf.add_institution(inst, props);
    }
}

/// Parse a TOML table for accounts and create Account structs
fn parse_accounts<'a, 'b>(accounts: &'a Map<String, Value>, conf: &'a mut Config<'b>) {
    for (acct, props) in accounts {
        conf.add_account(acct, props);
    }
}

/// Attempt to load and parse the config file into our Config struct.
/// If a file cannot be found, return a default Config.
/// If we find a file but cannot parse it, panic
pub fn parse<'a>(path: &Path) -> Config<'a> {
    // placeholder for config string contents
    let mut config_str = String::new();
    // default to be returned if no file found
    let default = Config {
        institutions: HashMap::<String, Institution>::new(),
        accounts: HashMap::<String, Account<'a>>::new(),
    };
    // config to be returned, otherwise
    let mut conf = Config {
        institutions: HashMap::new(),
        accounts: HashMap::new(),
    };

    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(_)  => {
            return default;
        }
    };

    // read file contents and assign to config_toml
    file.read_to_string(&mut config_str)
        .unwrap_or_else(|err| panic!("Error while reading config: [{}]", err));

    let config_toml = match config_str.parse() {
        Ok(Value::Table(s)) => s,
        _ => panic!("Error while parsing config: improperly formed Table")
    };
    // parse institutions
    if let Some(Value::Table(table)) = config_toml.get("Institutions") {
        parse_institutions(table, &mut conf);
    }
    // parse accounts
    if let Some(Value::Table(table)) = config_toml.get("Accounts") {
        parse_accounts(table, &mut conf);
    }
    
    return conf;
}
