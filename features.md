### Ripress Feature Catalog

This document enumerates all features implemented in the codebase, organized by area. It is derived from the current source under `src/` and aims to be exhaustive.

## Routing and Composition

- **Express-like application (`app::App`)**

  - Route registration via `get`, `post`, `put`, `delete`, `patch`, `head`, `options` for string paths.
  - Internally uses `types::RouterFns` trait to add routes into an in-memory `Routes` map.
  - Route handlers are async functions of signature `(HttpRequest, HttpResponse) -> HttpResponse`.
  - All routes are compiled into a `routerify` `Router` at `listen()` time.

- **Sub-routers (`router::Router`)**

  - Constructed with a `base_path` (e.g., `/api`).
  - Supports the same `get/post/...` API via `RouterFns`.
  - `register(&mut App)` mounts all router routes on `base_path` by prefixing each path.

- **HTTP methods supported**

  - `GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS` (see `types::HttpMethods`).

- **Route parameters**
  - Parameters extracted by `routerify` are injected into `HttpRequest.params`.
  - Rich accessors in `req::route_params::RouteParams`: `get`, `get_parsed<T>`, `get_int`, `get_uint`, `get_or_default`, `get_or_parse_default`, `contains`, `names`, `len`, `is_empty`, `iter`, `into_map`.
  - Convenience: `id()` for integer ids, `slug()` for string slugs.
  - Strong error typing via `ParamError` with friendly `Display`.

## Middleware System

- **Registration and execution**

  - `App::use_middleware(path, middleware)` registers pre-routing middleware with an optional path prefix (defaults to `/`).
  - Middlewares run in the order added. Path matching uses exact or prefix-with-`/` semantics.
  - Middleware signature: `Fn(HttpRequest, HttpResponse) -> FutMiddleware` returning `(HttpRequest, Option<HttpResponse>)`.
  - Returning `Some(HttpResponse)` short-circuits the pipeline; `None` continues to next middleware/handler.
  - `helpers::exec_middleware` performs conversion to/from Hyper and applies path filtering.
  - Middlewares can read/modify the request and set `RequestData` for downstream use.

- **Built-in middlewares (`middlewares`)**
  - `cors::cors(config)`
    - Adds CORS headers to all matching requests (`Access-Control-Allow-Origin`, `-Methods`, `-Headers`).
    - Optional `Access-Control-Allow-Credentials`.
    - Auto-handles `OPTIONS` preflight by responding `200 OK` and short-circuiting.
    - Configurable via `CorsConfig { allowed_origin, allowed_methods, allowed_headers, allow_credentials }` with defaults.
  - `logger::logger(config)`
    - Configurable booleans to log method, path, and duration.
    - Computes and prints timing per request; non-blocking and always continues.
  - `file_upload::file_upload(upload_dir)`
    - Handles binary uploads and browser `multipart/form-data` (extracts and saves the first file part).
    - Ensures upload directory exists; generates UUID-based filename; detects extension via `infer`.
    - Saves file asynchronously with `tokio::fs`.
    - Injects request data keys: `uploaded_file`, `uploaded_file_path`, and optional `original_filename`.
    - Logs errors to stderr and continues without short-circuiting on failures.

## Static File Serving

- `App::static_files(mount_path, fs_root)` enables a static file server.
- Mounts a `GET {mount_path}/*` route using `hyper-staticfile::Static` behind the scenes.
- Path rewriting strips the mount prefix so `/static/index.html` maps to `{fs_root}/index.html`.
- Adds `Cache-Control: public, max-age=86400` and `X-Served-By: hyper-staticfile`.
- Manual conditional handling for `If-None-Match` to return `304 Not Modified` when ETag matches.

## Request Model (`req::HttpRequest`)

- **Fields**

  - `params: RouteParams` — dynamic route parameters.
  - `query: QueryParams` — query-string parameters with multi-value support.
  - `origin_url: Url` — scheme and authority derived from URI or `Host` header.
  - `method: HttpMethods`
  - `ip: IpAddr` — from `X-Forwarded-For` or defaults to `127.0.0.1`.
  - `path: String`
  - `protocol: String` — from `x-forwarded-proto` with default `http`.
  - `xhr: bool` — when `X-Requested-With: XMLHttpRequest`.
  - `is_secure: bool` — `protocol == "https"`.
  - `headers: RequestHeaders` — case-insensitive multi-value request headers.
  - `cookies: HashMap<String, String>` — parsed from `Cookie` header.
  - `data: RequestData` — byte-efficient key/value storage set by middleware.
  - `body: RequestBody` — typed body content.

- **Cookies**

  - `get_cookie(name) -> Option<&String>`.

- **Middleware data**

  - `set_data(key, value)`, `get_data(key) -> Option<String>`, `get_all_data() -> &RequestData`.

- **Body content detection and access**

  - Content type is determined via `determine_content_type` (MIME-based) with support for:
    - JSON (`application/json` and `+json` subtypes)
    - Form (`application/x-www-form-urlencoded`)
    - Multipart form (`multipart/form-data`) — initially treated as `FORM`, but raw bytes are preserved as `BINARY` when building the body so upload middleware can parse it.
    - Text (`text/*` and XML variants mapped to text)
    - Binary (default/fallback)
  - Accessors:
    - `is(RequestBodyType)`
    - `bytes() -> Result<&[u8], String>` when `BINARY`
    - `json<T: DeserializeOwned + Serialize>() -> Result<T, String>`
    - `text() -> Result<&str, String>`
    - `form_data() -> Result<&FormData, String>`

- **Conversion to/from Hyper**
  - `from_hyper_request(&mut Request<Body>) -> Result<HttpRequest, hyper::Error>` reads headers, params, cookies, body, method, query, and constructs a typed `HttpRequest`.
  - `to_hyper_request() -> Result<Request<Body>, Box<dyn Error>>` writes headers, cookies, query, and encodes the current typed body (JSON/text/form/binary) and request data into extensions.

### Request Headers (`req::request_headers::RequestHeaders`)

- Case-insensitive multimap of header name to values.
- Methods: `insert`, `append`, `get`, `get_all`, `contains_key`, `remove`, `keys`, `len`, `is_empty`, `iter`, `iter_all`.
- Conveniences: `content_type`, `authorization`, `user_agent`, `accept`, `host`, `x_forwarded_for`, `accepts_json()`, `accepts_html()`.

### Query Parameters (`req::query_params::QueryParams`)

- Multi-value query param storage with parsing from query string and builders.
- Getters: `get`, `get_all`, `get_parsed<T>`, `get_all_parsed<T>`, typed helpers (`get_int`, `get_i64`, `get_uint`, `get_bool`, `get_float`).
- Utilities: `get_or_default`, `contains`, `has_value`, `names`, `len`, `is_empty`, `iter`, `iter_all`, `into_map`, `remove`.
- Common patterns: `page()`, `limit()`, `offset()`, `search_query()`, `sort()`, `sort_direction()`.
- Pretty `Display` and `Index` support.
- Strong errors via `QueryParamError` with detailed context.

### Route Parameters (`req::route_params::RouteParams`)

- See above in Routing; adds rich parsing, validation and convenience.

### Request Data (`req::request_data::RequestData`)

- High-performance byte-based map (`ByteKey` -> `Vec<u8>`) designed for middleware data.
- Insert variants: `insert(&[u8], &[u8])`, `insert_owned(Vec<u8>, Vec<u8>)`.
- Accessors: `get(&[u8]) -> Option<String>`, `remove(&[u8]) -> Option<Vec<u8>>`, `contains_key`, `len`, `is_empty`.
- Iteration: `iter()`, `keys()`, `values()`, `IntoIterator` for owned pairs.
- Utilities: `from_map(HashMap)`, `byte_size()`, `shrink_to_fit()`.

### Request Body Types (`req::body`)

- `RequestBody`, `RequestBodyType` and `RequestBodyContent` represent typed content: `JSON`, `TEXT`, `FORM`, `BINARY`, `EMPTY`.
- `FormData`
  - Wrapper over `HashMap<String,String>` with `insert/get/get_or/get_mut/as_map/keys/values/len/is_empty/remove/contains_key/from_map/iter/clear/retain/extend/append/to_query_string/from_query_string/from_comma_separated`.
  - Conversions and `Index` implementation.
- `TextData`
  - Safe wrapper over bytes with charset tracking and UTF-8 validation.
  - Constructors: `new(String)`, `from_bytes(Vec<u8>)`, `from_bytes_with_limit`, `from_raw_bytes(Vec<u8>, Option<String>)`.
  - Accessors: `as_str`, `as_str_lossy`, `into_string`, `into_string_lossy`, `as_bytes`, `_as_bytes_mut (crate)`, `into_bytes`.
  - Metrics/utilities: `len_bytes`, `len_chars`, `is_empty`, `charset/set_charset`, `is_valid_utf8`, `lines`, `trim`, `contains`, `split`, `truncate_bytes`, `truncated_bytes`.

## Response Model (`res::HttpResponse`)

- **Status**

  - Builder-style setters: `ok`, `created`, `accepted`, `no_content`, `bad_request`, `unauthorized`, `forbidden`, `not_found`, `method_not_allowed`, `conflict`, `internal_server_error`, `not_implemented`, `bad_gateway`, `service_unavailable`, and arbitrary `status(u16)`.

- **Body**

  - `text(T: Into<String>)`, `json<T: Serialize>(T)`, `html(&str)`, `bytes(Bytes)`.

- **Headers**

  - `set_header(name, value)` for arbitrary headers.
  - Rich `ResponseHeaders` builder available on `HttpResponse.headers`:
    - Content headers: `content_type`, `content_length`, `location`, `json`, `html`, `text`, `xml`, `attachment`, `inline`.
    - Caching: `cache_control`, `no_cache`, `etag`, `last_modified`.
    - Security: `frame_options`, `no_sniff`, `xss_protection`, `hsts`, `csp`, `security_headers`, `powered_by`, `remove_powered_by`.
    - CORS helpers: `cors_allow_origin`, `cors_allow_methods`, `cors_allow_headers`, `cors_allow_credentials`, `cors_simple`, plus builder-style `with_*` variants.

- **Cookies**

  - `set_cookie(name, value, CookieOptions)` with defaults (HttpOnly=true, Secure=true, SameSite=None, Path="/").
  - `clear_cookie(name)` removes and emits an expired cookie.

- **Redirects**

  - `redirect(url)` (302) and `permanent_redirect(url)` (301).

- **Streaming**

  - `write(stream)` accepts `Stream<Item = Result<Bytes, E>>` and marks response as streaming.
  - Sets headers for chunked transfer, disables caching, and uses `text/event-stream` with `Connection: keep-alive` for SSE-style streams.

- **Conversion to Hyper**
  - `to_responder()` builds a `hyper::Response<Body>`, including all headers and cookie set/clear handling.

### Response Status (`res::response_status::StatusCode`)

- Enum with common statuses plus `Custom(u16)`.
- Conversions: `as_u16()`, `from_u16(u16)`.
- Introspection: `is_success()`, `is_redirection()`, `is_client_error()`, `is_server_error()`, `is_informational()`.
- `canonical_reason()` and user-friendly `Display`.

## Server Engine (`App::listen`)

- Builds a `routerify::Router` with registered pre-middlewares and routes per HTTP method.
- Attaches static-file route if configured.
- Adds a central error handler mapping internal `ApiError` to `HttpResponse`.
- Binds and serves with `hyper::Server` on `127.0.0.1:{port}` and runs the provided startup callback.

## Helpers and Types

- `helpers::exec_middleware` — converts between Hyper and internal request/response for middleware and enforces path prefix matching.
- `helpers::get_all_query` — serializes `QueryParams` back into a query string.
- `types::RouterFns` — shared trait that powers `App` and `Router` route registration; includes `add_route`, verb helpers, and lookup.
- `types::Handler/Fut` — boxed async handler type definitions.
- `types::ResponseContentBody/ResponseContentType` — internal response content tagging.
- `app::api_error::ApiError` — unified error wrapper; conversions from `hyper::Error` and `Infallible` with safe fallback to `500`.

## Dependency-powered Capabilities

- `tokio` for async runtime, filesystem, IO utilities.
- `hyper` for HTTP server implementation and streaming bodies.
- `routerify` for ergonomic route/middleware composition and param extraction.
- `hyper-staticfile` for static file serving.
- `cookie` for cookie building and serialization.
- `mime` for robust content-type parsing.
- `infer` for file type detection in uploads.
- `uuid` for unique file name generation.
- `bytes`, `futures`, `serde`, `serde_json`, `serde_urlencoded`, `url`, `urlencoding` utilities for data handling.

## Public API Methods Index

This section lists public methods and functions intended for application developers, organized by module and type.

### app::App

- `App::new()`
- `App::use_middleware(path: Into<Option<&'static str>>, middleware)`
- `App::static_files(mount_path: &'static str, fs_root: &'static str)`
- `App::listen(port: u16, cb: impl FnOnce() + 'static)`
- Via `types::RouterFns` (implemented for `App`):
  - `get(path, handler)`
  - `options(path, handler)`
  - `post(path, handler)`
  - `put(path, handler)`
  - `delete(path, handler)`
  - `head(path, handler)`
  - `patch(path, handler)`

### router::Router

- `Router::new(base_path: &'static str)`
- `Router::register(app: &mut App)`
- Via `types::RouterFns` (implemented for `Router`):
  - `get(path, handler)`
  - `options(path, handler)`
  - `post(path, handler)`
  - `put(path, handler)`
  - `delete(path, handler)`
  - `head(path, handler)`
  - `patch(path, handler)`

### types::RouterFns (trait)

- `add_route(method, path, handler)`
- `get(path, handler)`
- `options(path, handler)`
- `post(path, handler)`
- `put(path, handler)`
- `delete(path, handler)`
- `head(path, handler)`
- `patch(path, handler)`

### context::HttpRequest (alias of `req::HttpRequest`)

- `HttpRequest::new()`
- `get_cookie(name) -> Option<&String>`
- `set_data(key, value)`
- `get_all_data() -> &RequestData`
- `get_data(key) -> Option<String>`
- `insert_form_field(key: &str, value: &str)`
- `is(RequestBodyType) -> bool`
- `bytes() -> Result<&[u8], String>`
- `json<T: DeserializeOwned + Serialize>() -> Result<T, String>`
- `text() -> Result<&str, String>`
- `form_data() -> Result<&FormData, String>`
### req::request_headers::RequestHeaders

- `RequestHeaders::new()`
- `insert(key, value)`
- `append(key, value)`
- `get(key) -> Option<&str>`
- `get_all(key) -> Option<&Vec<String>>`
- `contains_key(key) -> bool`
- `remove(key) -> Option<Vec<String>>`
- `content_type() -> Option<&str>`
- `authorization() -> Option<&str>`
- `user_agent() -> Option<&str>`
- `accept() -> Option<&str>`
- `host() -> Option<&str>`
- `x_forwarded_for() -> Option<&str>`
- `accepts_json() -> bool`
- `accepts_html() -> bool`
- `keys() -> impl Iterator<Item=&String>`
- `len() -> usize`
- `is_empty() -> bool`
- `iter() -> impl Iterator<Item=(&String, &str)>`
- `iter_all() -> impl Iterator<Item=(&String, &Vec<String>)>`

### req::query_params::QueryParams

- `QueryParams::new()`
- `from_map(HashMap<String, String>) -> Self`
- `from_query_string(&str) -> Self`
- `insert(key, value)`
- `append(key, value)`
- `get(name) -> Option<&str>`
- `get_all(name) -> Option<&Vec<String>>`
- `get_parsed<T: FromStr>() -> Result<T, QueryParamError>`
- `get_all_parsed<T: FromStr>() -> Result<Vec<T>, QueryParamError>`
- `get_int(name) -> Result<i32, QueryParamError>`
- `get_i64(name) -> Result<i64, QueryParamError>`
- `get_uint(name) -> Result<u32, QueryParamError>`
- `get_bool(name) -> Result<bool, QueryParamError>`
- `get_float(name) -> Result<f64, QueryParamError>`
- `get_or_default<T: FromStr>(name, default: T) -> T`
- `contains(name) -> bool`
- `has_value(name) -> bool`
- `names() -> impl Iterator<Item=&String>`
- `len() -> usize`
- `is_empty() -> bool`
- `iter() -> impl Iterator<Item=(&String, &str)>`
- `iter_all() -> impl Iterator<Item=(&String, &Vec<String>)>`
- `into_map(self) -> HashMap<String, String>`
- `remove(name) -> Option<Vec<String>>`
- Common patterns:
  - `page() -> i32`
  - `limit() -> i32`
  - `offset() -> i32`
  - `search_query() -> Option<&str>`
  - `sort() -> Option<&str>`
  - `sort_direction() -> SortDirection`
  - `filters() -> HashMap<String, Vec<String>>`
  - `is_truthy(name) -> bool`

### req::route_params::RouteParams

- `RouteParams::new()`
- `from_map(HashMap<String, String>) -> Self`
- `insert(key, value)`
- `get(name) -> Option<&str>`
- `get_parsed<T: FromStr>() -> Result<T, ParamError>`
- `get_int(name) -> Result<i32, ParamError>`
- `get_uint(name) -> Result<u32, ParamError>`
- `get_or_default<T: FromStr>(name, default: T) -> T`
- `get_or_parse_default<T: FromStr>(name, default: T) -> Result<T, ParamError>`
- `contains(name) -> bool`
- `names() -> impl Iterator<Item=&String>`
- `len() -> usize`
- `is_empty() -> bool`
- `iter() -> impl Iterator<Item=(&String, &String)>`
- `into_map(self) -> HashMap<String, String>`
- `extract<F: FnOnce(&Self) -> Result<(), Vec<ParamError>>>(f) -> Result<(), Vec<ParamError>>`
- Convenience:
  - `id() -> Result<i32, ParamError>`
  - `slug() -> Option<&str>`

### req::request_data::RequestData

- `RequestData::new()`
- `with_capacity(capacity: usize) -> Self`
- `insert(key: impl AsRef<[u8]>, value: impl AsRef<[u8]>)`
- `insert_owned(key: Vec<u8>, value: Vec<u8>)`
- `get(key) -> Option<String>`
- `remove(key) -> Option<Vec<u8>>`
- `contains_key(key) -> bool`
- `len() -> usize`
- `is_empty() -> bool`
- `clear()`
- `iter() -> impl Iterator<Item=(&[u8], &[u8])>`
- `keys() -> impl Iterator<Item=&[u8]>`
- `values() -> impl Iterator<Item=&[u8]>`
- `from_map(HashMap<K,V>) -> Self`
- `byte_size() -> usize`
- `shrink_to_fit()`

### req::origin_url::Url

- `as_str() -> &str`
- `value() -> &String`

### req::body::FormData

- `FormData::new()`
- `with_capacity(usize)`
- `insert(key, value) -> Option<String>`
- `get(key) -> Option<&str>`
- `get_or(key, default) -> &str`
- `get_mut(key) -> Option<&mut String>`
- `as_map() -> &HashMap<String,String>`
- `keys() -> impl Iterator<Item=&str>`
- `values() -> impl Iterator<Item=&str>`
- `len() -> usize`
- `is_empty() -> bool`
- `remove(key) -> Option<String>`
- `contains_key(key) -> bool`
- `from_map(HashMap<String,String>) -> Self`
- `iter() -> impl Iterator<Item=(&str,&str)>`
- `clear()`
- `retain(F)`
- `extend(iter)`
- `append(key, value)`
- `to_query_string() -> String`
- `from_query_string(&str) -> Result<Self, String>`
- `from_comma_separated(&str) -> Result<Self, String>`

### req::body::TextData

- `TextData::new(String)`
- `from_bytes(Vec<u8>) -> Result<Self, TextDataError>`
- `from_bytes_with_limit(Vec<u8>, usize) -> Result<Self, TextDataError>`
- `from_raw_bytes(Vec<u8>, Option<String>) -> Self`
- `as_str() -> Result<&str, TextDataError>`
- `as_str_lossy() -> Cow<str>`
- `into_string(self) -> Result<String, TextDataError>`
- `into_string_lossy(self) -> String`
- `as_bytes() -> &[u8]`
- `into_bytes(self) -> Vec<u8]`
- `len_bytes() -> usize`
- `len_chars() -> Result<usize, TextDataError>`
- `is_empty() -> bool`
- `charset() -> Option<&str>`
- `set_charset(String)`
- `is_valid_utf8() -> bool`
- `lines() -> Result<std::str::Lines, TextDataError>`
- `trim() -> Result<&str, TextDataError>`
- `contains(&str) -> Result<bool, TextDataError>`
- `split(delim: &str) -> Result<std::str::Split<'_, &str>, TextDataError>`
- `truncate_bytes(max_len: usize)`
- `truncated_bytes(max_len: usize) -> Self`

### context::HttpResponse (alias of `res::HttpResponse`)

- `HttpResponse::new()`
- Status builders: `ok`, `created`, `accepted`, `no_content`, `bad_request`, `unauthorized`, `forbidden`, `not_found`, `method_not_allowed`, `conflict`, `internal_server_error`, `not_implemented`, `bad_gateway`, `service_unavailable`, `status(u16)`
- Body: `text`, `json`, `html`, `bytes`
- Headers: `set_header(name, value)`
- Cookies: `set_cookie(name, value, CookieOptions)`, `clear_cookie(name)`
- Redirects: `redirect(url)`, `permanent_redirect(url)`
- Streaming: `write(stream)`
- Header builder field: `headers: ResponseHeaders` (see below)

### res::response_headers::ResponseHeaders

- Constructors: `new()`, `from_static_map(HashMap<&'static str, &'static str>)`
- Setters: `insert`, `append`, `remove`
- Getters: `get`, `get_all`, `contains_key`, `keys`, `len`, `is_empty`, `iter`, `iter_all`, `to_map`, `to_header_lines`
- Content helpers: `content_type`, `content_length`, `location`, `json`, `html`, `text`, `xml`, `attachment`, `inline`
- Caching: `cache_control`, `no_cache`, `etag`, `last_modified`
- Security: `frame_options`, `no_sniff`, `xss_protection`, `hsts`, `csp`, `security_headers`, `powered_by`, `remove_powered_by`
- CORS: `cors_allow_origin`, `cors_allow_methods`, `cors_allow_headers`, `cors_allow_credentials`, `cors_simple`
- Builder-style: `with_header`, `with_content_type`, `with_cors`, `with_security`

### res::response_status::StatusCode

- `as_u16() -> u16`
- `from_u16(u16) -> StatusCode`
- `is_success() -> bool`
- `is_redirection() -> bool`
- `is_client_error() -> bool`
- `is_server_error() -> bool`
- `is_informational() -> bool`
- `canonical_reason() -> &'static str`

### middlewares

- `middlewares::cors::cors(config: Option<CorsConfig>)`
  - `CorsConfig::default()` and fields: `allowed_origin`, `allowed_methods`, `allowed_headers`, `allow_credentials`
- `middlewares::logger::logger(config: Option<LoggerConfig>)`
  - `LoggerConfig::default()` and fields: `method`, `path`, `duration`
- `middlewares::file_upload::file_upload(upload_dir: Option<&str>)`

## Notes and Limitations

- File upload middleware extracts and saves only the first file in `multipart/form-data` and does not parse non-file fields.
- Logger prints to stdout; for production consider structured logging.
- Static file serving sets caching headers and handles `If-None-Match` manually, relying on `hyper-staticfile` for ETag generation.
- `listen()` binds to `127.0.0.1`; external exposure requires changes if needed.
