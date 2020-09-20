# Quill

Quill: query your bills and track your financial statements.
Check for statements that you've downloaded from your bank, service provider, or other company that issues regular statements.

## Installation

```shell
git clone https://github.com/jrhawley/quill.git
cd quill
cargo build --release
```

## Usage

```shell
quill 0.1.0
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
    list    List accounts and institutions from the config file
    next    List upcoming bills from all accounts
```

## Examples

```shell
$ cd quill/examples
Chequing: [2020-09-18]
Phone: [2020-09-17]
Savings: [2020-09-18]

$ quill next
 Account  | Institution    | Next Bill
----------+----------------+------------
 Chequing | My Bank        | 2020-10-15
 Phone    | Phone Provider | 2020-10-19
 Savings  | My Bank        | 2020-12-15

$ quill list
Configuration file:
        config.toml

Institutions:
        My Bank
        Phone Provider

Accounts:
        Chequing
        Phone
        Savings
```

## Design implementation

See [this blog post](https://jrhawley.github.io/2020/09/19/financial-statements-quill/) for details about the motivation and design implementation of Quill.
