# Quill

Query all your bills and accounts to check on your financial statements.
Check for statements that you've downloaded from your bank, service provider, or other company that issues regular statements.

![TUI demo](examples/demo.gif)

## Installation

On Windows, Linux, or macOS, install with [Cargo](https://doc.rust-lang.org/cargo/).

```shell
git clone https://github.com/jrhawley/quill.git
cd quill
cargo install --path .
```

## Usage

```shell
> quill -h
Query all your bills and accounts to check on your financial statements.

USAGE:
    quill [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config <CONF>    The statement configuration file
```

## How it works

See [this blog post](https://jrhawley.github.io/2020/09/19/financial-statements-quill) for details about the motivation and design implementation of Quill.

