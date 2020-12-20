use crate::models::{account::Account, date::Date, statement};
use bat::{Input, PagingMode, PrettyPrinter};
use statement::Statement;

fn print_pair(d: &Date, s: &Option<Statement>) -> String {
    let mut line = d.to_string();
    if s.is_some() {
        let stmt = s.clone().unwrap();
        let stmt_name = stmt.path().file_name().unwrap().to_str().unwrap();
        line.push('\t');
        line.push_str(stmt_name);
    }
    return line;
}

pub fn log_account_dates(acct: &Account) {
    let dates: Vec<String> = acct
        .match_statements()
        .iter()
        .map(|(d, s)| print_pair(d, s))
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
