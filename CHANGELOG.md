# Changelog

## [0.2.3] - 2025-03-12

### Added

- Post Requests now work.
- req.json::<Struct>(), req.text(), req.form_data() methods.

### Fixed

- Text responses can now handle both String and &str.

## [0.2.2] - 2025-03-12

### Added

- Params and Query parsing.

### Changed

- Codebase refactoring for better maintainability.

### Fixed

- Resolved serialization issues and improved stability under high load.

## [0.2.1] - 2025-03-11

### Added

- Only GET Requests are supported for now.
