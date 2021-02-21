# Changelog

## Unreleased

### Added

### Changed

### Deprecated

### Fixed

### Removed

### Security

## 0.2.0 - 2021-21-02

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