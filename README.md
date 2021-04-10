# Quill

Query all your bills and accounts to check on your financial statements.
Check for statements that you've downloaded from your bank, service provider, or other company that issues regular statements.

## Installation

```shell
git clone https://github.com/jrhawley/quill.git
cd quill
cargo install --path .
```

## Usage

```shell
Query all your bills and accounts to check on your statements.

USAGE:
    quill.exe [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config <FILE>    The statement configuration file [default: config.toml]

SUBCOMMANDS:
    help    Prints this message or the help of the given subcommand(s)
    list    List accounts and institutions from the config file [aliases: ls]
    log     Show a history log from a given account [aliases: l]
    next    List upcoming bills from all accounts [aliases: n]
    prev    List most recent bills from all accounts [aliases: p]
```

## Examples

```shell
$ cd quill/examples
$ quill
Chequing: [2020-09-18]
Phone: [2020-09-17]
Savings: [2020-09-18]

$ quill next
 Account  | Institution    | Next Bill
----------+----------------+------------
 Chequing | My Bank        | 2020-10-15
 Phone    | Phone Provider | 2020-10-19
 Savings  | My Bank        | 2020-12-15

$ quill prev
 Account  | Institution    | Most Recent Bill
----------+----------------+------------------
 Savings  | My Bank        | 2020-07-15
 Chequing | My Bank        | 2020-09-15
 Phone    | Phone Provider | 2020-10-05

$ quill list
Configuration file:
        examples/config.toml

Institutions:
        My Bank
        Phone Provider

Accounts:
        Chequing
        Phone
        Savings
```

## How it works

See [this blog post](https://jrhawley.github.io/2020/09/19/financial-statements-quill) for details about the motivation and design implementation of Quill.

