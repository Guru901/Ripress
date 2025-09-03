# Changelog

## [1.8.3] - 2025-09-4

Fixed cyclic dependency issues when using with wynd

## [1.8.2] - 2025-09-4

Updated websocket versions

## [1.8.1] - 2025-09-3

Fixed nightly feature gates

## [1.8.0] - 2025-09-3

Added `app.use_wynd` method
Added wynd middleware to the app

## [1.7.5] - 2025-08-22

Added a few impls to make it work with wynd

## [1.7.4] - 2025-08-22

Edited somethings for wynd integration

## [1.7.3] - 2025-08-22

Removed unnecessary dependencies

## [1.7.2] - 2025-08-22

Added more tests
Fixed wrong header insertion in Shield middleware

## [1.7.1] - 2025-08-22

Fixed failing tests and multiple static file serving bugs

## [1.7.0] - 2025-08-22

Added `res.send_file` method
`app.static_files` now returns `Result<(), &'static str>`

## [1.6.0] - 2025-08-22

Added Shield middleware
Added `app.use_shield` method

## [1.5.0] - 2025-08-22

Added Compression middleware
Added `app.use_compression` method

## [1.4.0] - 2025-08-22

Added Body Size Limit middleware
Added `app.use_body_limit` method

## [1.3.1] - 2025-08-22

File upload middleware configuration now works

## [1.3.0] - 2025-08-22

Added Rate Limiter middleware
Added `app.use_cors` method
Added `app.use_logger` method
Added `app.use_rate_limiter` method
Made app methods chainable

## [1.2.0] - 2025-08-21

Added logger middleware configuration options

## [1.1.3] - 2025-08-20

Finally fixed

## [1.1.2] - 2025-08-20

Fixed broken file upload middleware

## [1.1.1] - 2025-08-20

Fixed multipart formdata parsing with no files
Added more tests

## [1.1.0] - 2025-08-19

Added file upload middleware
Added support for `multipart/form-data` requests
Added support for binary data

## [1.0.1] - 2025-08-14

This release finalizes the API for the 1.x series. Several request/response
methods have been made type-safe, and some method names have been changed or
removed for consistency.

### Breaking Changes

- `req.form_data()` now returns `FormData` instead of `HashMap<String, String>`.
- `req.text()` now returns `TextData` instead of `String`.
- `req.ip` now returns `IpAddr` instead of `String`.
- `req.origin_url` now returns `Url` instead of `String`.
- `req.query_params` now returns `QueryParams` instead of `HashMap`.
- `req.route_params` now returns `RouteParams` instead of `HashMap`.
- `req.header` now returns `RequestHeaders` instead of `HashMap`.
- Removed `req.get_headers()` → use `req.headers.get()` instead.
- `res.headers` now returns `ResponseHeaders` instead of `HashMap`.
- Removed `res.set_header()` → set headers via `res.headers.insert()` or equivalent.
- Public API audited: all request/response accessors now use strongly typed wrappers.

### Added

- `RequestHeaders` struct for request headers.
- `RequestData` struct for generic request extensions.
- `FormData` struct for parsed form fields.
- `TextData` struct for plain-text bodies.
- `QueryParams` struct for parsed query parameters.
- `RouteParams` struct for path parameters.
- `ResponseHeaders` struct for response headers.
- `ResponseStatus` enum for HTTP status codes.

### Removed

- `req.get_header()` — use `req.headers.get()`.
- `res.set_header()` — use the new `ResponseHeaders` API.

### Internal / Other

- Middleware API stabilized for 1.x.
- Documentation updated with new types and examples.

## [1.0.1] - Yanked

## [0.6.1] - 2025-07-26

- Readded static files serving capabilities

## [0.6.0] - 2025-07-26

- some internal changes to make it perform better
- added `res.permanent_redirect` method
- removed unnecessary code
- refactored a lot

## [0.5.1] - 2025-04-01

- Added `res.head` method

## [0.5.0] - 2025-03-22

- Added `res.redirect` method
- Added **streaming responses**

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
