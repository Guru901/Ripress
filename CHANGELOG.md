# Changelog

## [2.1.1] - 2025-12-19

- Added `app.use_pre_middlewares`
- Added `app.use_post_middlewares`
- Added `middlewares!` macro

## [2.1.0] - 2025-12-5

- Added http2 support by default for the server
- Added `app.enable_http2` method
- Added `app.http2_config` method
- Added http2 config struct with sane defaults
- Fixed a bug in content type detection
- Added unit tests

## [2.0.5] - 2025-12-4

- Optimized middleware application
- Improve error handling for form and JSON data parsing in HttpRequest
  - Added logging for parsing errors, defaulting to empty form data or null JSON as appropriate.
  - Improved robustness of request body processing by ensuring errors are reported without crashing the application.
- Added benchmarks

## [2.0.4] - 2025-12-3

- Added `app.host` method to change the host, defaults to `0.0.0.0`
- Internal refactoring
- Performance optimizations

## [2.0.3] - 2025-11-20

- Request parsing is 2x faster than before
- Request conversion is 28% faster than before
- Response headers is 57% faster than before
- "/ Hello world" tests went from 147k requests/sec to 200k requests/sec
  - Not a test that would mean much but just showing the difference

## [2.0.2] - 2025-11-17

- Performance Improvements

  - Request Handling

    - Optimized header parsing with pre-allocated capacity and single-pass construction (#133)

      - Headers are now built in a single pass with no intermediate HashMap allocations
      - Added RequestHeaders::with_capacity() for efficient pre-allocation

    - Optimized cookie extraction with inline parsing

      - Replaced private cookie extraction function with inline parsing directly from headers
      - Eliminates unnecessary function calls and intermediate allocations

    - Improved IP address extraction

      - More robust single-pass parsing of X-Forwarded-For header
      - Better fallback handling to 127.0.0.1

    - Optimized body handling across all content types
      - Minimized allocations for FORM, MULTIPART_FORM, JSON, TEXT, and BINARY types
      - Uses from_slice for JSON parsing
      - Reuses pre-serialized strings where applicable

  - Response Handling

    - Optimized JSON response serialization

      - JSON is now serialized once and reused, reducing duplicate serialization overhead

    - Improved Set-Cookie header handling

      - Set-Cookie headers are now processed separately and aggregated efficiently
      - Prevents header overwrites and ensures all cookies are preserved

    - Fixed streaming response handling

      - Properly sets transfer-encoding: chunked header
      - Removes Content-Length header for streamed responses
      - Safe concatenation of stream chunks

    - Query Parameters
      - Pre-allocated HashMap in QueryParams::from_map() (#133)
      - Allocates the internal HashMap with exact capacity, reducing reallocation overhead

- Internal Improvements
  - Refactored middleware execution for clearer control flow (#133)
  - Explicit request construction in exec_pre_middleware None branch
  - Removed unnecessary blank line in exec_post_middleware

## [2.0.0] - 2025-11-15

- Feature-gated middleware: use_logger() and use_compression() now require their respective features to be enabled

  - Enable with ripress = { version = "2.0.0", features = ["logger", "compression"] }

- Added three new optional features for modular compilation:

  - compression - Enables compression middleware (requires flate2)
  - file-upload - Enables file upload middleware (requires uuid)
  - logger - Enables logging middleware (requires tracing)

- Updated hyper from 1.7.0 to 1.8.1
- Fixed unnecessary mutability in exec_pre_middleware helper
- Enhanced test coverage with server readiness checks
- Added feature-gated test compilation for modular testing

## [1.10.0] - 2025-10-19

- Upgraded to hyper 1.7.0
- Removed `Routerify` for a custom version `Routerify-ng` that's built by me to work with hyper 1.7.0

## [1.9.12] - 2025-10-19

- Deprecated `router.register(&mut app)` in favor of `app.router(router)`

## [1.9.11] - 2025-10-04

- Added graceful shutdown with `app.with_graceful_shutdown()`

## [1.9.10] - 2025-10-03

- Updated API route setup to use Arc::clone for handler cloning, improving memory management and making Arc usage more explicit

- Fixed bug with route params panics with middlewares

## [1.9.9] - 2025-09-27

Moved docs to https://ripress.vercel.app

## [1.9.8] - 2025-09-25

Docs improved

## [1.9.7] - 2025-09-19

- Added RipressError::new(), message(), and kind() methods for better error management
- Comprehensive Module Docs: Added detailed documentation across all major modules
- Usage Examples: Extensive code examples for routing, middleware, and handlers
- API References: Updated all documentation to reflect new import paths
- Feature Explanations: Added explanations for router grouping, composition, and versioning
- Import Reorganization: Cleaned up import statements across the codebase

## [1.9.6] - 2025-09-18

### Added

- Extended cookie attributes:

  - domain (optional domain for cookies)
  - max_age (optional max age for cookie expiration)
  - expires (optional explicit expiration timestamp)

- Refined cookie builder pattern for safer and more complete cookie construction

### Changed

- Modularized cookie functionality: moved all cookie-related types to res::response_cookie module for clearer separation of concerns
- Updated documentation examples to use new module paths

### Internal

- Improved cookie handling in both streaming and non-streaming response paths
- Strengthened cookie attribute preservation during response building
- Updated tests to reflect new cookie module paths

## [1.9.5] - 2025-09-17

- Global Error Type from `ripress::error::RipressError`

## [1.9.4] - 2025-09-16

- Internal refactoring

## [1.9.3] - 2025-09-15

- Improved README

## [1.9.2] - 2025-09-13

- Used tracing instead of println! for logging

## [1.9.1] - 2025-09-11

- Fixed incorrect docs for `res.set_cookie`

## [1.9.0] - 2025-09-11

### Added

- **Post-middleware support**: Added `use_post_middleware` method to execute middleware after route handlers
- **Middleware types**: Introduced `MiddlewareType::Pre` and `MiddlewareType::Post` enum variants
- **Enhanced middleware execution**: Middleware now executes in proper order (pre → route → post)
- **Post-middleware documentation**: Comprehensive documentation for pre and post middleware usage

### Changed

- **Middleware execution flow**: Updated middleware execution to support both pre and post processing
- **Built-in middleware types**: Logger and compression middleware now use post-middleware by default
- **Documentation updates**: Updated API reference and guides to include post-middleware examples
- **Set Cookie**: res.set_cookie now takes option of cookie config and when none is passed default is set

### Technical Details

- Added `MiddlewareType` enum to distinguish between pre and post middleware
- Updated `App::listen` method to handle different middleware types appropriately
- Enhanced middleware execution helpers to support post-middleware processing
- Added comprehensive tests for post-middleware functionality

## [1.8.9] - 2025-09-10

- Removed more things to reduce the bundle size

## [1.8.8] - 2025-09-10

- Removed tests and docs from published crate

## [1.8.7] - 2025-09-06

- Fixed wynd integration not working with custom middlewares
- Added integration tests for testing wynd integrations

## [1.8.6] - 2025-09-06

- Fixed a bug with wynd integration method

## [1.8.5] - 2025-09-06

- Removed unused code
- Added contributing guidelines

## [1.8.4] - 2025-09-04

- Added Tests

## [1.8.3] - 2025-09-04

- Fixed cyclic dependency issues when using with wynd

## [1.8.2] - 2025-09-04

- Updated websocket versions

## [1.8.1] - 2025-09-03

- Fixed nightly feature gates

## [1.8.0] - 2025-09-03

- Added `app.use_wynd` method
- Added wynd middleware to the app

## [1.7.5] - 2025-08-22

- Added a few impls to make it work with wynd

## [1.7.4] - 2025-08-22

- Edited somethings for wynd integration

## [1.7.3] - 2025-08-22

- Removed unnecessary dependencies

## [1.7.2] - 2025-08-22

- Added more tests
- Fixed wrong header insertion in Shield middleware

## [1.7.1] - 2025-08-22

- Fixed failing tests and multiple static file serving bugs

## [1.7.0] - 2025-08-22

- Added `res.send_file` method
- `app.static_files` now returns `Result<(), &'static str>`

## [1.6.0] - 2025-08-22

- Added Shield middleware
- Added `app.use_shield` method

## [1.5.0] - 2025-08-22

- Added Compression middleware
- Added `app.use_compression` method

## [1.4.0] - 2025-08-22

- Added Body Size Limit middleware
- Added `app.use_body_limit` method

## [1.3.1] - 2025-08-22

- File upload middleware configuration now works

## [1.3.0] - 2025-08-22

- Added Rate Limiter middleware
- Added `app.use_cors` method
- Added `app.use_logger` method
- Added `app.use_rate_limiter` method
- Made app methods chainable

## [1.2.0] - 2025-08-21

- Added logger middleware configuration options

## [1.1.3] - 2025-08-20

- Finally fixed

## [1.1.2] - 2025-08-20

- Fixed broken file upload middleware

## [1.1.1] - 2025-08-20

- Fixed multipart formdata parsing with no files
- Added more tests

## [1.1.0] - 2025-08-19

- Added file upload middleware
- Added support for `multipart/form-data` requests
- Added support for binary data

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

- Increased Code Coverage
- Improved Error Handling

## [0.3.1] - 2025-03-15

### Added

- Request Methods

  - Added req.is_secure method
  - Added req.get_protocol method

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

## [0.3.0] - 2025-03-14

**Note:** Version 0.2.4 was an accidental release. Those changes are documented here under 0.3.0.

### Added

- Request Methods

  - Added req.is method
  - Added req.get_method method
  - Added req.get_origin_url method
  - Added req.get_path method
  - Added req.get_cookie method
  - Added req.ip method
  - Added req.get_header method

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
