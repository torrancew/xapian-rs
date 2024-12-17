# Changelog

Starting with v0.2.0, all notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

N/A

## [0.2.1] - 2024-12-16

### Added

- `Operator` is now part of the public API

## [0.2.0] - 2024-08-03

### Added

- Support for custom `FieldProcessor` implementations
- Support for query expansions via the Xapian `ESet` API
- A unified `NativeRangeProcessor` that wraps upstream Xapian's built-in `RangeProcessor`
  implementations

### Removed

- `DateRangeProcessor` (use `NativeRangeProcessor`)
- `NumberRangeProcessor` (use `NativeRangeProcessor`)
- `RangeProcessor` (use `NativeRangeProcessor`)

## [0.1.0] - 2024-07-09

### Added

- Initial project release
- Basic index & query support
- Support for user-provided MatchSpy and MatchDecider implementations
- Support for arbitrary types in Xapian document slots

[unreleased]: https://github.com/torrancew/xapian-rs/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/torrancew/xapian-rs/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/torrancew/xapian-rs/compare/tag/v0.1.0...v0.2.0
[0.1.0]: https://github.com/torrancew/xapian-rs/releases/tag/v0.1.0
