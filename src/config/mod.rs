use home::home_dir;
use kronos::{step_by, Grain, Grains, LastOf, NthOf, Shim};
use std::collections::HashMap;
use std::env::{self, current_dir};
use std::fmt::Display;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use toml::map::Map;
use toml::Value;

use crate::models::account::Account;
use crate::models::date::Date;
use crate::models::institution::Institution;

pub struct Config<'a> {
    // absolute path of the config file
    path: PathBuf,
    // institution information
    institutions: HashMap<String, Institution>,
    // account information
    accounts: HashMap<String, Account<'a>>,
}

impl<'a> Config<'a> {
    /// Attempt to load and parse the config file into our Config struct.
    /// If a file cannot be found, return a default Config.
    /// If we find a file but cannot parse it, panic
    pub fn new(path: &Path) -> Config<'a> {
        // placeholder for config string contents
        let mut config_str = String::new();
        // default to be returned if no file found
        let default = Config {
            // this forces an absolute path if none is given
            path: current_dir()
                .unwrap()
                .canonicalize()
                .unwrap()
                .join("config.toml"),
            institutions: HashMap::<String, Institution>::new(),
            accounts: HashMap::<String, Account<'a>>::new(),
        };
        // config to be returned, otherwise
        let mut conf = Config {
            path: PathBuf::from(path),
            institutions: HashMap::new(),
            accounts: HashMap::new(),
        };

        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(_) => {
                return default;
            }
        };

        // read file contents and assign to config_toml
        file.read_to_string(&mut config_str)
            .unwrap_or_else(|err| panic!("Error while reading config: [{}]", err));

        let config_toml = match config_str.parse() {
            Ok(Value::Table(s)) => s,
            _ => panic!("Error while parsing config: improperly formed Table"),
        };
        // parse institutions
        if let Some(Value::Table(table)) = config_toml.get("Institutions") {
            parse_institutions(table, &mut conf);
        }
        // parse accounts
        if let Some(Value::Table(table)) = config_toml.get("Accounts") {
            parse_accounts(table, &mut conf);
        }
        conf
    }

    /// Get the path of the config file
    /// By `new` implementation, it is assured that this is an absolute path
    pub fn path(&self) -> &Path {
        &self.path.as_path()
    }

    /// Get the HashMap of institutions in the configuration
    pub fn institutions(&self) -> &HashMap<String, Institution> {
        // return required here becuase of the pointer
        return &self.institutions;
    }

    /// Get the list of institution names in the configuration, sorted by name
    pub fn institutions_sorted(&self) -> (Vec<&str>, Vec<&str>) {
        // collect institution names
        let mut v = self
            .institutions()
            .iter()
            .map(|(k, _)| k.as_str())
            .collect::<Vec<&str>>();
        // sort before returning
        v.sort();
        // create sorted list of institutions
        let v_names = (&v)
            .iter()
            .map(|&k| self.institutions().get(k).unwrap().name())
            .collect();
        return (v, v_names);
    }
    /// Get the list of accounts in the configuration
    pub fn accounts(&self) -> &HashMap<String, Account<'a>> {
        // return required here becuase of the pointer
        return &self.accounts;
    }

    /// Get the list of account names in the configuration, sorted by name
    pub fn accounts_sorted(&self) -> (Vec<&str>, Vec<&str>) {
        // collect account keys
        let mut v = self
            .accounts()
            .iter()
            .map(|(k, _)| k.as_str())
            .collect::<Vec<&str>>();
        // sort before returning
        v.sort();
        // create list of account names, sorted by the keys
        let v_names = (&v)
            .iter()
            .map(|&k| self.accounts().get(k).unwrap().name())
            .collect();
        return (v, v_names);
    }
    /// Add a new account to the configuration
    pub fn add_account(&mut self, key: &str, props: &toml::Value) {
        // extract name, if available
        let name = match props.get("name") {
            Some(Value::String(n)) => n,
            _ => panic!("No name for account"),
        };
        // extract and lookup corresponding institution
        let inst = match props.get("institution") {
            Some(Value::String(i)) => {
                // look up institution `i` in `conf` and return its reference
                self.institutions().get(i).unwrap().name()
            }
            _ => panic!("No appropriate name for institution"),
        };

        // extract statement file name format
        let fmt = match props.get("statement_fmt") {
            Some(Value::String(f)) => f,
            _ => panic!("No statement name format for account"),
        };

        // extract directory containing statements
        let dir: PathBuf = match props.get("dir") {
            Some(Value::String(p)) => {
                // if path is relative, convert to absolute path with folder containing the config file
                let path = Path::new(p);
                if path.is_relative() {
                    self.path() // get the path of the config file
                        .parent() // get its parent directory
                        .unwrap()
                        .join(path) // join the relative path of the account dir
                        .canonicalize() // force it to absolute
                        .unwrap()
                } else {
                    path.to_path_buf()
                }
            }
            _ => panic!("No directory for account"),
        };

        // extract first statement date
        let stmt_first = match props.get("first_date") {
            Some(Value::Datetime(d)) => {
                Date::parse_from_str(&d.to_string(), "%Y-%m-%dT%H:%M:%S%:z").unwrap()
            }
            _ => panic!("No date for first statement"),
        };

        // extract statement period
        let period = match props.get("statement_period") {
            Some(Value::Array(p)) => {
                // check if using LastOf or Nth of to generate dates
                let mut is_lastof = false;
                if p.len() != 4 {
                    panic!("Improperly formatted statement period");
                }
                let nth: usize = match &p[0] {
                    Value::Integer(n) => {
                        if *n < 0 {
                            is_lastof = true;
                        }
                        (*n).abs() as usize
                    }
                    _ => panic!("Non-integer for `nth` statement period"),
                };
                let mth: usize = match &p[3] {
                    Value::Integer(m) => *m as usize,
                    _ => panic! {"Non-integer for `mth` statement period"},
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
                        "Millenium" => Grains(Grain::Millenium),
                        _ => Grains(Grain::Day),
                    };
                } else {
                    panic!("Non-string for `x` statement period");
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
                        "Millenium" => Grains(Grain::Millenium),
                        _ => Grains(Grain::Day),
                    };
                } else {
                    panic!("Non-string for `y` statement period");
                }
                let y_step = step_by(y, mth);
                // return the TimeSequence object
                if is_lastof {
                    Shim::new(LastOf(nth, x, y_step))
                } else {
                    Shim::new(NthOf(nth, x, y_step))
                }
            }
            _ => panic!("Improperly formatted statement period"),
        };
        // create account and push to conf
        // can't use serialization here for the entire account
        // because we have a more complex relationship between the Account struct and its components
        let a = Account::new(name, inst, stmt_first, period, fmt, dir);
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

    /// Query configuration by the account name or key
    pub fn query_account(&self, s: &str) -> Option<&Account> {
        // check `s` against both keys and names
        let (acct_keys, acct_names) = self.accounts_sorted();
        match acct_keys.contains(&s) {
            true => Some(self.accounts().get(s).unwrap()),
            false => match acct_names.iter().position(|&a| a == s) {
                Some(idx) => {
                    let acct_key = acct_keys[idx];
                    self.accounts().get(acct_key)
                }
                None => None,
            },
        }
    }

    /// Query configuration by the institution name or key
    pub fn query_institution(&self, s: &str) -> Option<&Institution> {
        // check `s` against both keys and names
        let (inst_keys, inst_names) = self.institutions_sorted();
        match inst_keys.contains(&s) {
            true => Some(self.institutions().get(s).unwrap()),
            false => match inst_names.iter().position(|&a| a == s) {
                Some(idx) => {
                    let inst_key = inst_keys[idx];
                    self.institutions().get(inst_key)
                }
                None => None,
            },
        }
    }
}

impl<'a> Display for Config<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Institutions: {:?}", self.institutions.values())
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

/// Check multiple locations for a configuration file and return the highest priority one
pub fn get_config_path() -> PathBuf {
    // check for `QUILL_CONFIG` environment variable
    match env::var("QUILL_CONFIG") {
        Ok(p) => PathBuf::from(p),
        Err(_) => {
            // check if $XDG_CONFIG_HOME is set
            let mut cfg_path = match env::var("XDG_CONFIG_HOME") {
                Ok(dir) => PathBuf::from(dir),
                // if not set, make it the default $HOME/.config
                Err(_) => {
                    if let Some(mut dir) = home_dir() {
                        dir.push(".config");
                        dir
                    } else {
                        PathBuf::new()
                    }
                }
            };

            // get config from within $XDG_CONFIG_HOME
            cfg_path.push("quill");
            cfg_path.push("config.toml");
            match cfg_path.exists() {
                true => cfg_path,
                false => PathBuf::from("config.toml"),
            }
        }
    }
}
