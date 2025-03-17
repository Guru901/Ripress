# Changelog

## [0.3.2] - 2025-03-15

### Added

- Response Methods

  - Added res.html method

- Increased Code Covergae
- Improved Error Handling

## [0.3.1] - 2025-03-15

### Added

- Request Methods

  - Added res.is_secure method
  - Added res.get_protocol method

- Response Methods

  - Added res.set_cookie method
  - Added res.clear_cookie method
  - Added res.set_header method
  - Added res.get_header method
  - Added res.set_content_type method

- App Methods

  - Added app.patch method
  - Added app.all method

- Added Integration tests
- Increased Code Coverage

### Changed

- Added a types module to make the codebase more readable and maintainable.

## Move the 0.2.4 changes under 0.3.0 and note that 0.2.4 was an accidental release.

## [0.2.4] - 2025-03-14

### Added

- Request Methods

  - Added res.is method
  - Added res.get_method method
  - Added res.get_origin_url method
  - Added res.get_path method
  - Added res.get_cookie method
  - Added res.ip method
  - Added res.get_header method

- Response Methods

  - Added various helpers for status codes
    - res.ok()
    - res.bad_request()
    - res.not_found()
    - res.internal_server_error()

- Added Docs
- Added Unit Tests

### Changed

- Added a types module to make the codebase more readable and maintainable.

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
