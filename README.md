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
Quill uses the [`dirs`](https://docs.rs/dirs/latest/dirs/) crate to achieve this, which follows the [XDG specifications](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html) on Linux, [Known Folder](https://docs.microsoft.com/en-us/previous-versions/windows/desktop/legacy/bb776911(v=vs.85)?redirectedfrom=MSDN) conventions on Windows, and [Standard Directories](https://developer.apple.com/library/archive/documentation/FileManagement/Conceptual/FileSystemProgrammingGuide/FileSystemOverview/FileSystemOverview.html#//apple_ref/doc/uid/TP40010672-CH2-SW6) on macOS.
Put another way:

| Operating system | Configuration location                                  |
| ---------------- | ------------------------------------------------------- |
| macOS            | `~/Users/Library/Application Support/quill/config.toml` |
| Linux            | `~/.config/quill/config.toml`                           |
| Windows          | `~\\AppData\\Roaming\\quill\\config.toml`               |

An example configuration file can be found in [`examples/`](examples/config.toml).

### Ignore statements

In the directory for an account whose statements you're checking, you can include a `.quillignore.toml` file with an array of dates and/or file names.
Example ignore files can be found in [`examples/`](examples/).
