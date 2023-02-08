//! Global account configuration details.

use crate::cli::CliOpts;
use anyhow::{bail, Context};
use quill_account::Account;
use quill_statement::{StatementCollection, ObservedStatement, IgnoreFile, ignorefile_path_from_dir};
use quill_utils::parse_toml_file;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use toml::{map::Map, Value};

/// Account and program configuration
#[derive(Debug)]
pub struct Config<'config> {
    /// Absolute path of the config file
    path: PathBuf,

    /// Account information
    accounts: HashMap<String, Account<'config>>,

    /// Ordered index of accounts
    account_order: Vec<String>,

    /// Fast-access number of accounts
    num_accounts: usize,

    /// Collection of account statements
    acct_stmts: StatementCollection,
}

impl<'config> Config<'config> {
    /// Get the path of the config file
    /// By `new` implementation, it is assured that this is an absolute path
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    /// Get the list of accounts in the configuration
    pub fn accounts(&self) -> &HashMap<String, Account<'config>> {
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
    fn parse_accounts(&mut self, accounts: &Map<String, Value>) -> anyhow::Result<()> {
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

    /// Retrieve an account key from some selected index.
    pub fn get_account_key(&self, selected_acct: usize) -> String {
        self.keys()[selected_acct].to_string()
    }

    /// Retrieve a mutable pointer to an account using its key.
    pub fn get_account(&self, acct_key: &str) -> Option<&Account> {
        self.accounts().get(acct_key)
    }

    /// Retrieve the statements for each account
    pub fn statements(&self) -> &StatementCollection {
        &self.acct_stmts
    }
    
    /// Retrieve a mutable pointer to the statements for each account
    pub fn mut_statements(&mut self) -> &mut StatementCollection {
        &mut self.acct_stmts
    }
    
    /// Retrieve an account key and statement from some selected indices.
    pub fn get_account_statement(&self, selected_acct: usize, selected_stmt: usize) -> (String, &ObservedStatement) {
        // get the key for the selected account
        let acct_name = self.get_account_key(selected_acct);
        
        // obtain the statement file
        let obs_stmt = self
            .statements()
            .get(&acct_name)
            .unwrap()
            .iter()
            .rev()
            .nth(selected_stmt)
            .unwrap();
        
        (acct_name, obs_stmt)
    }
    
    /// Find all statements for each account
    pub fn scan_account_statements(&self) -> anyhow::Result<StatementCollection> {
        StatementCollection::try_from(self)
    }

    /// Add a date to an [`Account`'s][quill_account::account::Account] ignore list.
    pub fn ignore_statement(&mut self, selected_acct: usize, selected_stmt: usize) -> anyhow::Result<()> {
        let acct_key = self.get_account_key(selected_acct);

        let date = {
            let (_, _obs_stmt) = self.get_account_statement(selected_acct, selected_stmt);
            _obs_stmt.statement().date().clone()
        };

        if let Some(acct) = self.get_account(&acct_key) {
            let mut new_ignored = acct.ignored().clone();
            new_ignored.push(&date);

            // create a `IgnoreFile` and parse it into a TOML string
            let new_ignore_file = IgnoreFile::from(&new_ignored);
            let ignore_file_toml = toml::to_string(&new_ignore_file)?;

            // write this to the account's ignore file
            let path = ignorefile_path_from_dir(acct.directory());
            let mut file = match path.exists() {
                true => File::open(&path)?,
                false => File::create(&path)?                
            };
            write!(file, "{}", ignore_file_toml)?;
        }
        
        // re-scan for the statements, since this should be updated now
        self.refresh_account_statements()
    }

    /// Update the HashMap of all statements for each account
    pub fn refresh_account_statements(&mut self) -> anyhow::Result<()> {
        let new_sc = self.scan_account_statements()?;
        self.acct_stmts = new_sc;

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
            acct_stmts: StatementCollection::new(),
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
            Some(Value::Table(table)) => {
                conf.parse_accounts(table)?;
                conf.refresh_account_statements()?;
            },
            Some(_) => bail!("Error parsing the `[Accounts]` table in configuration file `{}`.", value.config().display()),
            None => bail!(
                "No `[Accounts]` table found in configuration file `{}`.\nPlease check the configuration and try again.",
                value.config().display(),
            )
        }

        Ok(conf)
    }
}
