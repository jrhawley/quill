use crate::models::account::Account;
use bat::{Input, PagingMode, PrettyPrinter};

pub fn log_account_dates(acct: &Account) {
    let dates: Vec<String> = acct
        .statement_dates()
        .iter()
        .map(|d| d.to_string())
        .collect();
    let date_input: Vec<Input> = dates
        .iter()
        .map(|d| Input::from_bytes(d.as_bytes()))
        .collect();
    // print statements dates and ignore any errors
    let _ = PrettyPrinter::new()
        .inputs(date_input)
        .paging_mode(PagingMode::QuitIfOneScreen)
        .print();
}
