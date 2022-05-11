# Changelog

## [0.7.2] - 2022-05-11

### Changed

- Moved `expand_tilde()` into the `quill_utils` crate
  - Added tests for the function

### Fixed

- Identified a bug where paths that did not exist would throw an unhelpful error
  - Checking if the directory first, before canonicalizing the path, fixed the issue to make error statements more clear

## [0.7.1] - 2022-05-10

### Added

- `Upcoming` tab to highlight when the next statement will be available for each account

## [0.7.0] - 2022-04-20

### Added

- Statements now support multiple periods
  - you can get statements on the 10th and 24th of each month, for example
  - previously, you'd have to specify something like once every two weeks, which would eventually drift away from the desired time sequence
  - this is specified in the `statement_period` part of the config file for an account
  - the new format is `[n, x, m, y]` where:
    - `n` is either an integer or an array of integers
    - `m` is an integer
    - `x` and `y` are strings

### Changed

- Updated colour scheme
  - previous colours were yellow and blue, used haphazardly; now all orange
  - now using gray, dark grey, and red text for available, ignored, and missing statements, respectively, in the `Log` tab

## [0.6.1] - 2022-02-19

### Added

- More error messages and error handling when parsing data

### Changed

- Statement file name formats must include extensions
  - previously, formats were partially matched file names
  - this led to mismatches with other files in the same folder, or similarly-named files with other extensions
- Internal refactoring into crates
  - making `quill-account` and `quill-statement` crates to make a clearer separation between library logic and TUI client
- New statement matching algorithm
  - addresses bugs with other files in the folder
  - also makes debugging much clearer

### Removed

- Removing ability to ignore files by a file path
  - this greatly simplifies ignorefile parsing
  - filtering by file name was inexact and didn't always work

## [0.6.0] - 2022-02-03

### Added

- Clearer error messages when bad data is parsed

### Changed

- Switched to the [`dirs` crate](https://docs.rs/dirs/latest/dirs/) for handling config file locations.
  - **macOS users will need to move their config files from `~/.config/quill/` to `$HOME/Library/Application Support/quill/`**.
- Configuration for statement periods have changed
  - **All users will need to update their config files to the new format**
  - The newly required format is `[n, x, m, y]` where `n` and `m` are integers and `x` and `y` are strings
  - This is different from the previous format of `[n, x, y, m]` (`m` and `y` are switched)
- Major refactoring of codebase.
  - `Account`s, `Statement`s, and other internal `struct`s are in their own libraries with clearer error messages

## [0.5.1] - 2021-11-15

### Changed

- Using 2021 edition of Rust instead of the 2018 edition.
  - This shouldn't affect anything in this codebase, but denoting it for future troubleshooting.

### Fixed

- Issues parsing ignore files where dates wouldn't match up properly with an account's expected statements.

## [0.5.0] - 2021-11-12

### Added

- Ignored statement functionality.
  - Using `.quillignore.toml` files places in the account's folder, you can specify certain dates or files that should count as ignored.
  - Quill will pretend that those statements are found, even if they don't exist.
  - See [this section](README.md#ignore-statements) of the README for details on how to write these files.

### Changed

- Configuration files should now describe their first statement dates using the ISO 8601 (i.e. `%Y-%m-%d` or `YYYY-MM-DD`) format.
  - This makes writing your configuration easier, but is a breaking change from previous versions.
- Large amounts of refactoring, but that shouldn't affect the end user.

### Fixed

- Accounts whose first statements are in the future no longer appear as "missing".

## [0.4.1] - 2021-10-15

### Changed

- No longer checking for `QUILL_CONFIG` environment variable.
- Only checking configuration directories for configuration files. See the [`dirs` documentation](https://docs.rs/dirs/4.0.0/dirs/fn.config_dir.html) for details on these locations.

## [0.4.0] - 2021-10-15

### Fixed

- Configuration files containing `~` characters in directory paths are now parsed correctly.

### Changed

- Removed the `Institution` struct, as it was redundant.
- Institutions are only referred to by name in the `Account` structs.
- Institutions are no longer required in the configuration.
- Various refactors.

## [0.3.3] - 2021-09-21

### Added

- Open PDFs or a file explorer by pressing `Enter` on the Log page

## [0.3.2] - 2021-08-17

### Added

- Friendly logging and warning for missing directories in the configuration

## [0.3.1] - 2021-08-09

### Changed

- Adding support for `XDG_CONFIG_HOME` config file parsing instead of relying on environment variables

## [0.3.0] - 2021-04-25

## Added

- Interactive terminal UI instead of paginated display
- Multiple tabs for interacting with different aspects of the configuration
- Arrow key/vi-style navigation

## [0.2.2] - 2021-04-10

### Changed

- No features, attempting automated builds

## [0.2.1] - 2021-03-07

### Fixed

- Previously excluded the most recent statement

## [0.2.0] - 2021-03-03

### Change

- Updating dependencies

### Fixed

- Previously not showing statement files passed the last expected available date

## [0.1.9] - 2021-01-17

### Changed

- Only parsing files that match a file statement format in the given folder, not all files

## [0.1.8] - 2020-12-29

### Fixed

- Safe handling of missing statement files
- Last, most recent, statement was previously not shown

## [0.1.7] - 2020-12-20

### Added

- `log` subcommand to show all statements for eac account
- Pagination with [`bat`](https://github.com/sharkdp/bat)

### Changed

- Pretty table formatting
- Better statement file date matching

## [0.1.6] - 2020-10-08

### Changed

- Internals of how jumps are handled
- No user-facing changes

## [0.1.5] - 2020-10-08

### Added

- Adding a `prev` subcommand to look at the previous statements for each account

## [0.1.4] - 2020-10-05

### Added

- Parse a configuration file from the `QUILL_HOME` environment variable

### Changed

- Custom implementation of `NthOf` to a `Shim` from Kronos
- Simplified account parsing from configuration files

## [0.1.3] - 2020-09-20

### Added

- Customized configuration file in TOML format
- Tracking accounts from the configuration file

## [0.1.2] - 2020-09-17

### Added

- `list` subcommand
- Pretty printing statements in a table

## [0.1.0] - 2020-09-15

- Initial release
