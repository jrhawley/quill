# Changelog

## [0.3.3] - 2020-09-21

### Added

- Open PDFs or a file explorer by pressing `Enter` on the Log page

## [0.3.2] - 2020-08-17

### Added

- Friendly logging and warning for missing directories in the configuration

## [0.3.1] - 2020-08-09

### Changed

- Adding support for `XDG_CONFIG_HOME` config file parsing instead of relying on environment variables

## [0.3.0] - 2020-04-25

## Added

- Interactive terminal UI instead of paginated display
- Multiple tabs for interacting with different aspects of the configuration
- Arrow key/vi-style navigation

## [0.2.2] - 2020-04-10

### Changed

- No features, attempting automated builds

## [0.2.1] - 2020-03-07

### Fixed

- Previously excluded the most recent statement

## [0.2.0] - 2020-03-03

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
