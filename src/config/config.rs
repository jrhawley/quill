//! Global account configuration details.

use anyhow::{bail, Context};
use quill_account::Account;
use quill_utils::parse_toml_file;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};
use toml::{map::Map, Value};

use crate::cli::CliOpts;

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

    /// Add a new account to the configuration
    pub fn add_account(&mut self, key: &str, props: &toml::Value) -> anyhow::Result<()> {
        // create account and push to conf
        // can't use serialization here for the entire account because there is
        // a more complex relationship between the Account struct and its
        // components
        let acct = Account::try_from(props)?;

        // update the account order with a binary search
        match self.account_order.binary_search(&key.to_string()) {
            Ok(_) => bail!(
                "Account key `{}` is duplicated. Please check your configuration file to ensure keys are unique.",
                &key
            ),
            Err(pos) => self.account_order.insert(pos, key.to_string()),
        };

        // insert the account object into the configuration
        self.accounts.insert(key.to_string(), acct);
        self.num_accounts += 1;

        Ok(())
    }

    /// Parse a TOML table for accounts and create Accounts
    fn parse_accounts<'b>(&mut self, accounts: &'b Map<String, Value>) -> anyhow::Result<()> {
        for (acct, props) in accounts {
            // add the account to the configuration
            // error out if any account isn't added properly
            self.add_account(acct, props).with_context(|| {
                format!(
                    "Error adding account `{}` with the following properties:\n{:#?}",
                    acct, props,
                )
            })?;
        }

        Ok(())
    }
}

impl TryFrom<CliOpts> for Config<'_> {
    type Error = anyhow::Error;

    fn try_from(value: CliOpts) -> anyhow::Result<Self, Self::Error> {
        if !value.config().exists() {
            bail!(
                "Configuration file `{}` does not exist.",
                value.config().display()
            );
        }

        // config to be returned, if parsed properly
        let mut conf = Self {
            path: value.config().to_path_buf(),
            accounts: HashMap::new(),
            account_order: Vec::new(),
            num_accounts: 0,
        };

        let config_str = parse_toml_file(value.config()).with_context(|| {
            format!(
                "Error reading contents of configuration file `{}`.\nPlease check the configuration and try again.",
                value.config().display()
            )
        })?;

        let config_toml = match config_str.parse() {
            Ok(Value::Table(s)) => s,
            Ok(_) => {
                bail!(
                    "Error parsing configuration file `{}`.\nPlease check the configuration and try again.",
                    value.config().display(),
                );
            }
            Err(e) => return Err(e).with_context(|| format!("Error parsing configuration file `{}`.\nPlease check the configuration and try again.", value.config().display())),
        };

        // parse accounts
        match config_toml.get("Accounts") {
            Some(Value::Table(table)) => conf.parse_accounts(table)?,
            Some(_) => bail!("Error parsing the `[Accounts]` table in configuration file `{}`.", value.config().display()),
            None => bail!(
                "No `[Accounts]` table found in configuration file `{}`.\nPlease check the configuration and try again.",
                value.config().display(),
            )
        }

        Ok(conf)
    }
}
