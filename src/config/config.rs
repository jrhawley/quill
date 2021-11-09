//! Global account configuration details.

use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::Display;
use std::fs::File;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use toml::Value;

use crate::{config::utils::parse_accounts, models::Account};

/// Account and program configuration
#[derive(Debug)]
pub struct Config<'a> {
    // absolute path of the config file
    path: PathBuf,

    // account information
    accounts: HashMap<String, Account<'a>>,

    // ordered index of accounts
    account_order: Vec<String>,

    // fast-access number of accounts
    num_accounts: usize,
}

impl<'a> Config<'a> {
    /// Attempt to load and parse the config file into our Config struct.
    /// If a file cannot be found, return a default Config.
    /// If we find a file but cannot parse it, panic
    pub fn new_from_path(path: &Path) -> Result<Config<'a>, Error> {
        // placeholder for config string contents
        let mut config_str = String::new();
        // config to be returned, if parsed properly
        let mut conf = Config {
            path: PathBuf::from(path),
            accounts: HashMap::new(),
            account_order: Vec::new(),
            num_accounts: 0,
        };

        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(e) => {
                return Err(Error::new(ErrorKind::NotFound, e));
            }
        };

        // read file contents and assign to config_toml
        match file.read_to_string(&mut config_str) {
            Ok(_) => {}
            Err(e) => return Err(Error::new(ErrorKind::InvalidData, e)),
        }

        let config_toml = match config_str.parse() {
            Ok(Value::Table(s)) => s,
            Ok(_) => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Error parsing config TOML table.",
                ))
            }
            Err(e) => return Err(Error::new(ErrorKind::InvalidInput, e)),
        };
        // parse accounts
        if let Some(Value::Table(table)) = config_toml.get("Accounts") {
            match parse_accounts(table, &mut conf) {
                Ok(_) => {}
                Err(e) => return Err(Error::new(ErrorKind::InvalidData, e)),
            };
        }

        Ok(conf)
    }

    /// Get the path of the config file
    /// By `new` implementation, it is assured that this is an absolute path
    pub fn path(&self) -> &Path {
        &self.path.as_path()
    }

    /// Get the list of accounts in the configuration
    pub fn accounts(&self) -> &HashMap<String, Account<'a>> {
        // return required here because of the pointer
        &self.accounts
    }

    /// Return the sorted account keys
    pub fn keys(&self) -> &Vec<String> {
        &self.account_order
    }

    /// Return the number of accounts in the configuration
    pub fn len(&self) -> usize {
        self.num_accounts
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
    pub fn add_account(&mut self, key: &str, props: &toml::Value) -> Result<(), Error> {
        // create account and push to conf
        // can't use serialization here for the entire account because there is
        // a more complex relationship between the Account struct and its
        // components
        let acct = Account::try_from(props)?;

        // update the account order with a binary search
        match self.account_order.binary_search(&key.to_string()) {
            Ok(_) => return Err(Error::new(
                ErrorKind::AlreadyExists,
                format!(
                    "Account key `{}` is duplicated. Please check your configuration file to ensure keys are unique.",
                    &key
                )
            )),
            Err(pos) => self.account_order.insert(pos, key.to_string())
        };

        // insert the account object into the configuration
        self.accounts.insert(key.to_string(), acct);
        self.num_accounts += 1;

        Ok(())
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
}

impl<'a> Display for Config<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Accounts: {:?}", self.accounts_sorted())
    }
}
