# Quill

Query all your bills and accounts to check on your financial statements.
Check for statements that you've downloaded from your bank, service provider, or other company that issues regular statements.

![TUI demo](examples/demo.gif)

## Installation

On Windows, Linux, or macOS, install with [Cargo](https://doc.rust-lang.org/cargo/).

```shell
cargo install --git https://github.com/jrhawley/quill.git
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

## Features

### Customized configuration

A configuration file will automatically be loaded from your user's application settings, if one exists.
Quill uses the [`dirs`](https://docs.rs/dirs/latest/dirs/) crate to achieve this, which follows the [expected conventions](https://docs.rs/dirs/latest/dirs/fn.config_dir.html) in each operating system.

| Operating system | Configuration location                                    |
| ---------------- | --------------------------------------------------------- |
| macOS            | `$HOME/Library/Application Support/quill/config.toml`     |
| Linux            | `$HOME/.config/quill/config.toml`                         |
| Windows          | `C:\\Users\\<User>\\AppData\\Roaming\\quill\\config.toml` |

An example configuration file can be found in [`examples/`](examples/config.toml).

### Ignore statements

In the directory for an account whose statements you're checking, you can include a `.quillignore.toml` file with an array of dates and/or file names.
Example ignore files can be found in [`examples/`](examples/).
