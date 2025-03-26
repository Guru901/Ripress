# Changelog

## [0.4.7] - 2025-03-22

- The WebSocket module has been removed from Ripress to keep it focused on HTTP.

## [0.4.6] - 2025-03-22

- Made changes to websocket api

## [0.4.5] - 2025-03-22

- Added `.send` method to WebSocket

## [0.4.4] - 2025-03-22

- Added WebSocket Support

## [0.4.3] - 2025-03-22

- Added router implementation for grouping routes

## [0.4.2] - 2025-03-22

- Added built in cors and logging middlewares
- Improved test coverage
- Added some crud examples

## [0.4.1] - 2025-03-20

- Fixed the middleware always being global

## [0.4.0] - 2025-03-20

### Added

- Added Middleware support
- Added set_data and get_data methods to HttpRequest used in middlewares

### Changes

- Listen method now takes an i32 as port and a closure as a callback

## [0.3.5] - 2025-03-18

Made helper testing functions private

## [0.3.4] - 2025-03-18

Removed Unnecessary Dependencies

## [0.3.3] - 2025-03-18

Readme fixed

## [0.3.2] - 2025-03-18

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
