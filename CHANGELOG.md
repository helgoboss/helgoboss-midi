# Changelog

## Unreleased

### Added

### Changed

### Deprecated

### Fixed

### Removed

- Removed `core-error` dependency because it sometimes leads to build issues. As a consequence, errors don't implement its
  Error trait in no-std environments.

### Security

## 0.3.3 - 2021-10-15

### Fixed

- Fixed compatibility with `std::error::Error`: Error types were not implementing the standard error trait
  before even when building with the `std` feature enabled. Now they do.

## 0.3.2 - 2021-09-23

### Fixed

- Fixed version numbers in documentation.

## 0.3.1 - 2021-09-23

### Fixed

- Fixed `ControllerNumber::is_parameter_number_message_controller_number` to include increment/decrement 
  controller numbers.

## 0.3.0 - 2021-09-23

### Added

- Added support for working with (N)RPN data increment/decrement messages.

### Changed

- Changed method signature of `PollingParameterNumberMessageScanner::feed` (now returns an
  array instead of just an option).

## 0.2.0 - 2021-02-21

### Added

- Added `PollingParameterNumberMessageScanner` for detecting of many more (N)RPN short message sequences.

### Changed

- Changed method signature of `ParameterNumberMessage::to_short_messages` (added data entry byte order parameter).

## 0.1.2 - 2020-12-06

### Fixed

- Fixed version references in `README.md` and `lib.rs`.

## 0.1.1 - 2020-12-06

### Added
- Added `no_std` support.
- Added `FromStr` implementation for all newtypes.
- Added Continuous Integration via GitHub Actions.

### Changed
- Improved panic messages.
- Improved documentation.

### Fixed
- Fixed bug that made it possible to construct an invalid newtype when using `TryFrom` with a negative value.
- Fixed bug that made it possible to deserialize to an invalid newtype via Serde.
- Fixed incorrect conditional compilation.

## 0.1.0 - 2020-05-07

- Initial release.