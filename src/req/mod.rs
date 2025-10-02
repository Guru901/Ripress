//! # HTTP Request Module
//!
//! This module provides the core [`HttpRequest`] struct and associated functionality for handling
//! incoming HTTP requests in the Ripress web framework. The HttpRequest struct serves as a
//! comprehensive representation of an HTTP request, providing convenient access to all request
//! data including headers, cookies, query parameters, route parameters, and request body content.
//!
//! ## Key Features
//!
//! - **Comprehensive Request Data**: Access to all aspects of the HTTP request
//! - **Type-Safe Body Parsing**: JSON, form data, text, and binary content support
//! - **Cookie Management**: Easy cookie retrieval and manipulation
//! - **Route Parameters**: Dynamic route parameter extraction
//! - **Query String Parsing**: Automatic query parameter parsing
//! - **Header Access**: Type-safe header manipulation
//! - **Client Information**: IP address, protocol, and user agent detection
//! - **Security Features**: XHR detection, secure connection identification
//! - **Middleware Integration**: Request data sharing between middleware and handlers
//!
//! ## Request Lifecycle
//!
//! The HttpRequest is created from incoming Hyper requests and flows through the middleware chain:
//!
//! ```text
//! 1. Raw HTTP Request (from client)
//!      ↓
//! 2. HttpRequest::from_hyper_request() - Parse and structure request data
//!      ↓
//! 3. Pre-middleware processing - Modify request, add data
//!      ↓
//! 4. Route handler execution - Main business logic
//!      ↓
//! 5. Post-middleware processing - Modify response
//! ```
//!
//! ## Basic Usage
//!
//! ```no_run
//! use ripress::app::App;
//! use ripress::types::RouterFns;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Deserialize, Serialize)]
//! struct CreateUserRequest {
//!     name: String,
//!     email: String,
//!     age: Option<u32>,
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut app = App::new();
//!
//!     // GET request with query parameters
//!     app.get("/users", |req, res| async move {
//!         let page = req.query.get("page").unwrap_or("1");
//!         let limit = req.query.get("limit").unwrap_or("10");
//!         
//!         println!("Fetching users: page {}, limit {}", page, limit);
//!         res.ok().json(serde_json::json!({
//!             "users": [],
//!             "page": page,
//!             "limit": limit
//!         }))
//!     });
//!
//!     // POST request with JSON body
//!     app.post("/users", |req, res| async move {
//!         match req.json::<CreateUserRequest>() {
//!             Ok(user_data) => {
//!                 println!("Creating user: {} ({})", user_data.name, user_data.email);
//!                 res.ok().json(serde_json::json!({
//!                     "message": "User created successfully",
//!                     "id": 123
//!                 }))
//!             }
//!             Err(e) => {
//!                 res.bad_request().json(serde_json::json!({
//!                     "error": "Invalid JSON data",
//!                     "details": e
//!                 }))
//!             }
//!         }
//!     });
//!
//!     // Route with parameters
//!     app.get("/users/:id", |req, res| async move {
//!         let user_id = req.params.get("id").unwrap_or("0");
//!         println!("Fetching user with ID: {}", user_id);
//!         
//!         res.ok().json(serde_json::json!({
//!             "user": {
//!                 "id": user_id,
//!                 "name": "John Doe"
//!             }
//!         }))
//!     });
//!
//!     app.listen(3000, || println!("Server running on http://localhost:3000")).await;
//! }
//! ```
//!
//! ## Request Body Types
//!
//! The HttpRequest supports multiple body content types with type-safe access methods:
//!
//! ### JSON Content
//! ```no_run
//! use ripress::context::HttpRequest;
//! use ripress::context::HttpResponse;
//! use serde::{Deserialize, Serialize};
//!
//! // Deserialize JSON directly into structs
//!
//! #[derive(Deserialize, Serialize)]
//! struct MyStruct;
//!
//! async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
//!     match req.json::<MyStruct>() {
//!         Ok(data) => { /* handle structured data */ }
//!         Err(e) => { /* handle parsing error */ }
//!     }
//!
//!     res.ok()
//! }
//! ```
//!
//! ### Form Data
//! ```no_run
//! use ripress::context::HttpRequest;
//! use ripress::context::HttpResponse;
//!
//! // Access form fields from application/x-www-form-urlencoded
//! async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
//!     match req.form_data() {
//!         Ok(form) => {
//!             let username = form.get("username").unwrap_or("");
//!             let password = form.get("password").unwrap_or("");
//!         }
//!         Err(e) => { /* handle error */ }
//!     }
//!
//!     res.ok()
//! }
//! ```
//!
//! ### Text Content
//! ```no_run
//! use ripress::context::HttpRequest;
//! use ripress::context::HttpResponse;
//!
//! // Get raw text content
//! async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
//!     match req.text() {
//!         Ok(text_content) => println!("Received text: {}", text_content),
//!         Err(e) => println!("Not text content: {}", e),
//!     }
//!
//!     res.ok()
//! }
//! ```
//!
//! ### Binary Data
//! ```no_run
//! use ripress::context::HttpRequest;
//! use ripress::context::HttpResponse;
//!
//! // Access raw binary data
//! async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
//!     match req.bytes() {
//!         Ok(binary_data) => {
//!             println!("Received {} bytes", binary_data.len());
//!             // Process binary data (file upload, image, etc.)
//!         }
//!         Err(e) => println!("Not binary content: {}", e),
//!     }
//!     res.ok().text("Binary data processed")
//! }
//! ```
//!
//! ## Client Information Access
//!
//! ```rust
//! use ripress::app::App;
//! use ripress::types::RouterFns;
//!
//! let mut app = App::new();
//!
//! app.get("/info", |req, res| async move {
//!     // Client IP address (considers X-Forwarded-For for proxies)
//!     println!("Client IP: {}", req.ip);
//!     
//!     // Protocol detection
//!     if req.is_secure {
//!         println!("Secure HTTPS connection");
//!     } else {
//!         println!("HTTP connection");
//!     }
//!     
//!     // AJAX request detection
//!     if req.xhr {
//!         println!("AJAX request detected");
//!     }
//!     
//!     // User agent
//!     if let Some(user_agent) = req.headers.get("user-agent") {
//!         println!("User Agent: {}", user_agent);
//!     }
//!     
//!     res.ok().text("Request info logged")
//! });
//! ```
//!
//! ## Advanced Usage Patterns
//!
//! ### Middleware Data Sharing
//! ```no_run
//! use ripress::app::App;
//! use ripress::types::RouterFns;
//!
//! let mut app = App::new();
//!
//! // In middleware
//! app.use_pre_middleware(None, |mut req, res| async move {
//!     // Add authentication data
//!     req.set_data("user_id", "12345");
//!     req.set_data("user_role", "admin");
//!     (req, None)
//! });
//!
//! // In route handler
//! app.get("/dashboard", |req, res| async move {
//!     if let Some(user_id) = req.get_data("user_id") {
//!         if let Some(role) = req.get_data("user_role") {
//!             println!("User {} with role {} accessing dashboard", user_id, role);
//!         }
//!     }
//!     res.ok().text("Dashboard")
//! });
//! ```
//!
//! ### Cookie Management
//! ```no_run
//! use ripress::app::App;
//! use ripress::types::RouterFns;
//!
//! let mut app = App::new();
//!
//! app.get("/profile", |req, res| async move {
//!     // Check for session cookie
//!     match req.get_cookie("session_id") {
//!         Some(session_id) => {
//!             println!("Authenticated session: {}", session_id);
//!             res.ok().text("Welcome back!")
//!         }
//!         None => {
//!             res.unauthorized().text("Please log in")
//!         }
//!     }
//! });
//! ```
//!
//! ### Content Type Detection
//! ```no_run
//! use ripress::req::body::RequestBodyType;
//! use ripress::app::App;
//! use ripress::types::RouterFns;
//!
//! let mut app = App::new();
//!
//! app.post("/upload", |req, res| async move {
//!     if req.is(RequestBodyType::JSON) {
//!         // Handle JSON upload
//!         match req.json::<serde_json::Value>() {
//!             Ok(data) => { /* process JSON */ }
//!             Err(e) => { /* handle error */ }
//!         }
//!     } else if req.is(RequestBodyType::BINARY) {
//!         // Handle binary upload
//!         match req.bytes() {
//!             Ok(data) => { /* process binary data */ }
//!             Err(e) => { /* handle error */ }
//!         }
//!     } else {
//!         return res.bad_request().text("Unsupported content type");
//!     }
//!     
//!     res.ok().text("Upload processed")
//! });
//! ```

#![warn(missing_docs)]

use crate::{
    helpers::{extract_boundary, get_all_query, parse_multipart_form},
    req::body::{FormData, RequestBody, RequestBodyContent, RequestBodyType, TextData},
    types::{HttpMethods, ResponseContentType},
};
use cookie::Cookie;
use hyper::{Body, Request, body::to_bytes, header::HOST};
use mime::Mime;
use routerify::{RequestInfo, ext::RequestExt};
use serde_json::Value;
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
};

#[cfg(feature = "with-wynd")]
use tokio::io::{AsyncRead, AsyncWrite};

/// A struct that represents the request headers.
/// And it's methods.
pub mod request_headers;

/// A struct that represents the origin url of the request.
/// And it's methods.
pub mod origin_url;

/// A struct that represents the query parameters of the request.
/// And it's methods.
pub mod query_params;

/// Structs that represents the body of the requests.
/// And it's methods.
pub mod body;

/// A struct that represents the route parameters of the request.
/// And it's methods.
pub mod route_params;

/// A struct that represents the request data of the request.
/// And it's methods.
pub mod request_data;

use request_data::RequestData;

use origin_url::Url;
use query_params::QueryParams;
use request_headers::RequestHeaders;
use route_params::RouteParams;

/// Represents an incoming HTTP request with comprehensive access to request data.
///
/// The HttpRequest struct provides methods to access and manipulate all aspects
/// of an HTTP request including headers, cookies, query parameters, route parameters,
/// and request body content.
///
/// ## Examples
///
/// Basic usage:
/// ```rust
/// use ripress::req::HttpRequest;
///
/// let req = HttpRequest::new();
/// println!("Method: {:?}", req.method);
/// println!("Path: {}", req.path);
/// println!("Client IP: {:?}", req.ip);
/// ```
///
/// Working with JSON body:
/// ```rust
/// use ripress::context::HttpRequest;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Deserialize, Serialize)]
/// struct User {
///     name: String,
///     age: u32
/// }
///
/// let req = HttpRequest::new();
/// if let Ok(user) = req.json::<User>() {
///     println!("User: {} is {} years old", user.name, user.age);
/// }
/// ```
///

#[derive(Clone, Debug)]
pub struct HttpRequest {
    /// Dynamic route parameters extracted from the URL.
    pub params: RouteParams,

    /// Query parameters from the request URL.
    pub query: QueryParams,

    /// The full URL of the incoming request.
    pub origin_url: Url,

    /// The HTTP method used for the request (e.g., GET, POST, PUT, DELETE).
    pub method: HttpMethods,

    /// The IP address of the client making the request.
    pub ip: IpAddr,

    /// The requested endpoint path.
    pub path: String,

    /// Protocol of the request (HTTP or HTTPs)
    pub protocol: String,

    /// Boolean indicating if the request was made with AJAX (XMLHttpRequest).
    pub xhr: bool,

    /// Boolean indicating if the request was made with Https
    pub is_secure: bool,

    /// The request's headers
    pub headers: RequestHeaders,

    /// The request's cookies
    pub(crate) cookies: HashMap<String, String>,

    /// The Data set by middleware in the request to be used in the route handler
    pub data: RequestData,

    /// The request body, which may contain JSON, text, or form data or binary data.
    pub(crate) body: RequestBody,
}

impl Default for HttpRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpRequest {
    /// Creates a new empty HTTP request instance.
    ///
    /// ## Returns
    ///
    /// Returns a new `HttpRequest` with default values.
    ///
    /// ## Example
    /// ```rust
    /// use ripress::req::HttpRequest;
    /// use ripress::types::HttpMethods;
    ///
    /// let req = HttpRequest::new();
    /// assert_eq!(req.method, HttpMethods::GET);
    /// ```

    pub fn new() -> Self {
        HttpRequest {
            origin_url: Url::new(""),
            params: RouteParams::new(),
            query: QueryParams::new(),
            method: HttpMethods::GET,
            ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            path: String::new(),
            protocol: String::new(),
            headers: RequestHeaders::new(),
            data: RequestData::new(),
            body: RequestBody {
                content: RequestBodyContent::EMPTY,
                content_type: RequestBodyType::EMPTY,
            },
            cookies: HashMap::new(),
            xhr: false,
            is_secure: false,
        }
    }

    /// Retrieves a cookie value by name.
    ///
    /// ## Arguments
    ///
    /// * `name` - The name of the cookie to retrieve
    ///
    /// ## Returns
    ///
    /// Returns `Some(&String)` with the cookie value if found, or `None` if not found.
    ///
    /// ## Example
    /// ```rust
    /// use ripress::context::HttpRequest;
    ///
    /// let req = HttpRequest::new();
    ///
    /// match req.get_cookie("session_id") {
    ///     Some(session) => println!("Session ID: {}", session),
    ///     None => println!("No session cookie found")
    /// }
    /// ```

    pub fn get_cookie(&self, name: &str) -> Option<&String> {
        self.cookies.get(name)
    }

    /// Adds data from the middleware into the request.
    ///
    /// ## Arguments
    ///
    /// * `key` - The key of the data to retrieve
    /// * `value` - The value of the data to retrieve
    ///
    /// ## Example
    /// ```
    /// let mut req = ripress::req::HttpRequest::new();
    /// req.set_data("id", "123");
    /// let id = req.get_data("id");
    /// println!("Id: {:?}", id);
    /// ```

    pub fn set_data<T: Into<String>>(&mut self, data_key: T, data_value: T) {
        self.data.insert(data_key.into(), data_value.into());
    }

    /// Returns all data stored in the request by the middleware.
    ///
    /// ## Returns
    ///
    /// Returns `&HashMap<String, String>` with the data.
    ///
    /// ## Example
    /// ```
    /// let req = ripress::req::HttpRequest::new();
    ///
    /// let data = req.get_all_data();
    ///
    /// println!("Data: {}", data);
    /// ```

    pub fn get_all_data(&self) -> &RequestData {
        &self.data
    }

    /// Returns data stored in the request by the middleware.
    ///
    /// ## Arguments
    ///
    /// * `key` - The key of the data to retrieve
    ///
    /// ## Returns
    ///
    /// Returns `Option<&String>` with the data value if found, or `None` if not found.
    ///
    /// ## Example
    /// ```
    /// let req = ripress::context::HttpRequest::new();
    /// let id = req.get_data("id");
    /// println!("Id: {:?}", id);
    /// ```

    pub fn get_data<T: Into<String>>(&self, data_key: T) -> Option<String> {
        self.data.get(&data_key.into())
    }

    /// Checks if the request body matches a specific content type.
    ///
    /// ## Arguments
    ///
    /// * `content_type` - The `RequestBodyType` to check against
    ///
    /// ## Returns
    ///
    /// Returns `true` if the content type matches, `false` otherwise.
    ///
    /// ## Example
    /// ```rust
    /// use ripress::req::HttpRequest;
    /// use ripress::req::body::RequestBodyType;
    ///
    /// let req = HttpRequest::new();
    /// if req.is(RequestBodyType::JSON) {
    ///     // Handle JSON content
    /// }
    /// ```

    pub fn is(&self, content_type: RequestBodyType) -> bool {
        self.body.content_type == content_type
    }

    /// Returns a read-only view of the raw request body when it is binary.
    ///
    /// Returns:
    /// - `Ok(&[u8])` when `content_type` is `RequestBodyType::BINARY`.
    /// - `Err("Invalid Binary Content")` if the internal variant does not match the declared type.
    /// - `Err(...)` describing the actual content type if it is not binary.
    ///
    /// ## Example
    /// ```no_run
    /// let req = ripress::context::HttpRequest::new();
    /// if let Ok(bytes) = req.bytes() {
    ///     // process bytes...
    /// }
    /// ```

    pub fn bytes(&self) -> Result<&[u8], String> {
        let body = &self.body;

        if body.content_type == RequestBodyType::BINARY {
            match &body.content {
                RequestBodyContent::BINARY(bytes) => Ok(bytes.as_ref()),
                RequestBodyContent::BinaryWithFields(bytes, _) => Ok(bytes.as_ref()),
                _ => Err(String::from("Invalid Binary Content")),
            }
        } else {
            Err(format!(
                "Wrong body type, expected binary and found {}",
                body.content_type.to_string(),
            ))
        }
    }

    /// Deserializes the request body as JSON into the specified type.
    ///
    /// ## Type Parameters
    ///
    /// * `J` - The type to deserialize into, must implement `DeserializeOwned`
    ///
    /// ## Returns
    ///
    /// Returns `Ok(J)` with the deserialized value if successful, or
    /// `Err(String)` with an error message if deserialization fails.
    ///
    /// ## Example
    /// ```rust
    /// use ripress::context::HttpRequest;
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Deserialize, Serialize)]
    /// struct LoginData {
    ///     username: String,
    ///     password: String
    /// }
    ///
    /// let req = HttpRequest::new();
    /// match req.json::<LoginData>() {
    ///     Ok(data) => println!("Login attempt for: {}", data.username),
    ///     Err(e) => println!("Invalid login data: {}", e)
    /// }
    /// ```

    pub fn json<J>(&self) -> Result<J, String>
    where
        J: serde::de::DeserializeOwned + serde::Serialize,
    {
        let body = &self.body;

        if body.content_type == RequestBodyType::JSON {
            if let RequestBodyContent::JSON(ref json_value) = body.content {
                match serde_json::from_value::<J>(json_value.clone()) {
                    Ok(serialized) => Ok(serialized),
                    Err(e) => Err(format!("Failed to deserialize JSON: {}", e)),
                }
            } else {
                Err(String::from("Invalid JSON content"))
            }
        } else {
            Err(String::from("Wrong body type"))
        }
    }

    /// Returns request's text body.
    ///
    /// ## Example
    /// ```no_run
    /// let req = ripress::req::HttpRequest::new();
    /// let text = req.text().unwrap();
    /// println!("text : {:?}", text);
    /// ```
    ///
    /// This function returns the text body of the request.
    /// Returns an `Result<String>`, where `Ok(String)` contains the body if it is valid text, or `Err(error)` if it is not.

    pub fn text(&self) -> Result<&str, String> {
        let body = &self.body;

        if body.content_type == RequestBodyType::TEXT {
            if let RequestBodyContent::TEXT(ref text_value) = body.content {
                let value = text_value.as_str();
                match value {
                    Ok(value) => Ok(value),
                    Err(err) => Err(err.to_string()),
                }
            } else {
                Err(String::from("Invalid text content"))
            }
        } else {
            Err(String::from("Wrong body type"))
        }
    }

    /// Returns request's form_data body.
    ///
    /// ## Example
    /// ```no_run
    /// let req = ripress::req::HttpRequest::new();
    /// // Let' say form data was sent as key=value and key2=value2
    /// let form_data = req.form_data().unwrap();
    /// println!("key = : {:?}", form_data.get("key"));
    /// println!("key2 = : {:?}", form_data.get("key2"));
    /// ```
    ///
    /// This function returns a HashMap of the form data.
    /// Returns an `Result<HashMap<String, String>>`, where `Ok(HashMap<String, String>)` contains the form_data if it is valid form data, or `Err(error)` if it is not.

    pub fn form_data(&self) -> Result<&FormData, String> {
        let body = &self.body;

        match body.content_type {
            RequestBodyType::FORM => {
                if let RequestBodyContent::FORM(form_data) = &body.content {
                    Ok(form_data)
                } else {
                    Err(String::from("Invalid form content"))
                }
            }
            RequestBodyType::BINARY => {
                // Check if this binary content also contains form fields
                if let RequestBodyContent::BinaryWithFields(_, form_data) = &body.content {
                    Ok(form_data)
                } else {
                    Err(String::from("Binary content without form fields"))
                }
            }
            _ => Err(String::from("Wrong body type")),
        }
    }

    /// Inserts a key-value pair into the request's form data.
    ///
    /// If the current body is not `FORM`, this will initialize an empty `FormData`
    /// and set the body's content type to `FORM` before inserting the field.
    /// This is useful for middlewares that wish to expose computed values through
    /// the `form_data()` API, such as attaching file upload metadata.
    pub fn insert_form_field(&mut self, key: &str, value: &str) {
        // Ensure body is of FORM type and contains a FormData map
        if self.body.content_type != RequestBodyType::FORM {
            self.body.content_type = RequestBodyType::FORM;
            self.body.content = RequestBodyContent::FORM(FormData::new());
        }

        if let RequestBodyContent::FORM(ref mut form_data) = self.body.content {
            form_data.insert(key.to_string(), value.to_string());
        }
    }

    pub(crate) fn set_param(&mut self, key: &str, value: &str) {
        self.params.insert(key.to_string(), value.to_string());
    }

    fn get_cookies_from_req(req: &Request<Body>) -> Vec<Cookie<'_>> {
        let mut cookies = Vec::new();

        if let Some(header_value) = req.headers().get("cookie") {
            if let Ok(header_str) = header_value.to_str() {
                for cookie_str in header_str.split(';') {
                    if let Ok(cookie) = Cookie::parse(cookie_str.trim()) {
                        cookies.push(cookie);
                    }
                }
            }
        }

        cookies
    }

    fn get_cookies_from_req_info(req: &RequestInfo) -> Vec<Cookie<'_>> {
        let mut cookies = Vec::new();

        if let Some(header_value) = req.headers().get("cookie") {
            if let Ok(header_str) = header_value.to_str() {
                for cookie_str in header_str.split(';') {
                    if let Ok(cookie) = Cookie::parse(cookie_str.trim()) {
                        cookies.push(cookie);
                    }
                }
            }
        }

        cookies
    }

    pub(crate) async fn from_hyper_request(
        req: &mut Request<hyper::body::Body>,
    ) -> Result<Self, hyper::Error> {
        let origin_url = match req.uri().authority() {
            Some(authority) => {
                let scheme = req.uri().scheme_str().unwrap_or("http");
                Url::new(format!("{}://{}", scheme, authority))
            }
            None => {
                let uri_string = req
                    .headers()
                    .get(HOST)
                    .and_then(|host| host.to_str().ok())
                    .map(|host| {
                        // Determine scheme (you might want to check for TLS context)
                        let scheme = "http"; // or "https" if using TLS
                        format!("{}://{}", scheme, host)
                    })
                    .unwrap_or(String::new());

                Url::new(uri_string)
            }
        };

        let query_string = req.uri().query().unwrap_or("");

        let queries = url::form_urlencoded::parse(query_string.as_bytes())
            .filter_map(|(key, value)| Some((key.to_string(), value.to_string())))
            .collect::<HashMap<String, String>>();

        let query = QueryParams::from_map(queries);

        let method = HttpMethods::from(req.method());

        let ip = req
            .headers()
            .get("X-Forwarded-For")
            .and_then(|val| val.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
            .unwrap_or(String::new());

        let ip = match ip.parse::<IpAddr>() {
            Ok(ip) => ip,
            Err(_) => IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        };

        let path = req.uri().path().to_string();

        let mut cookies_map = HashMap::new();
        let cookies = Self::get_cookies_from_req(&req);

        cookies.iter().for_each(|cookie| {
            let (name, value) = (cookie.name(), cookie.value());
            cookies_map.insert(name.to_string(), value.to_string());
        });

        let mut headers: HashMap<String, String> = HashMap::new();

        req.headers().iter().for_each(|(key, value)| {
            if let Ok(header_value) = value.to_str() {
                headers.insert(key.to_string(), header_value.to_string());
            }
        });

        let headers = RequestHeaders::_from_map(headers);

        let params = RouteParams::new();

        let mut data = RequestData::new();

        if let Some(ext_data) = req.extensions().get::<RequestData>() {
            data = ext_data.clone();
        }

        let content_type = req
            .headers()
            .get(hyper::header::CONTENT_TYPE)
            .and_then(|val| val.to_str().ok())
            .map(determine_content_type_request)
            .unwrap_or(RequestBodyType::EMPTY);

        let protocol = req
            .headers()
            .get("x-forwarded-proto")
            .and_then(|val| val.to_str().ok())
            .unwrap_or("http")
            .to_string();

        let request_body = match content_type {
            RequestBodyType::FORM => {
                // Read the full body bytes first, then branch depending on content subtype
                let body_bytes = to_bytes(req.body_mut()).await;

                match body_bytes {
                    Ok(bytes) => {
                        let body_string = std::str::from_utf8(&bytes).unwrap_or("").to_string();
                        match FormData::from_query_string(&body_string) {
                            Ok(fd) => RequestBody::new_form(fd),
                            Err(_e) => {
                                // Prefer logging via `log`/`tracing` instead of printing.
                                RequestBody::new_form(FormData::new())
                            }
                        }
                    }
                    Err(err) => return Err(err),
                }
            }
            RequestBodyType::MultipartForm => {
                let body_bytes = to_bytes(req.body_mut()).await;

                match body_bytes {
                    Ok(bytes) => {
                        // Get content type header to extract boundary
                        let content_type_header = req
                            .headers()
                            .get(hyper::header::CONTENT_TYPE)
                            .and_then(|val| val.to_str().ok())
                            .unwrap_or_default();

                        let boundary = if content_type_header
                            .to_lowercase()
                            .contains("multipart/form-data")
                        {
                            extract_boundary(&content_type_header)
                        } else {
                            None
                        };

                        // Parse multipart/form-data using the same logic as the middleware
                        let (fields, file_parts) = if let Some(boundary) = boundary {
                            parse_multipart_form(&bytes, &boundary)
                        } else {
                            // If not multipart, try to parse as form data using the same method
                            let body_string = std::str::from_utf8(&bytes).unwrap_or("").to_string();
                            match FormData::from_query_string(&body_string) {
                                Ok(fd) => {
                                    // Convert FormData to fields vector for consistency
                                    let form_fields = fd
                                        .iter()
                                        .map(|(k, v)| (k.to_string(), v.to_string()))
                                        .collect::<Vec<(String, String)>>();
                                    (form_fields, Vec::new())
                                }
                                Err(_e) => (Vec::new(), Vec::new()),
                            }
                        };

                        // Convert fields back to FormData
                        let mut form_data = FormData::new();
                        for (key, value) in fields {
                            form_data.insert(key, value);
                        }

                        // INTELLIGENT DECISION:
                        // - Always extract and make text fields accessible via form_data()
                        // - If there are file parts, also preserve raw bytes as BINARY for middleware processing
                        // - If no file parts, just use FORM content
                        if !file_parts.is_empty() {
                            // Has files: preserve raw bytes for middleware AND make text fields accessible
                            // We'll set the body as BINARY but also insert the text fields into form_data
                            // This way both the middleware can process files AND form fields are accessible
                            RequestBody::new_binary_with_form_fields(bytes, form_data)
                        } else {
                            // No files: just use form data
                            RequestBody::new_form(form_data)
                        }
                    }
                    Err(err) => return Err(err),
                }
            }
            RequestBodyType::JSON => {
                let body_bytes = to_bytes(req.body_mut()).await;
                let body_json = match body_bytes {
                    Ok(bytes) => {
                        let s = match std::str::from_utf8(&bytes) {
                            Ok(s) => s,
                            Err(_) => "",
                        };
                        match serde_json::from_str::<serde_json::Value>(s) {
                            Ok(json) => json,
                            Err(_) => Value::Null,
                        }
                    }
                    Err(err) => return Err(err),
                };

                RequestBody::new_json(body_json)
            }
            RequestBodyType::TEXT => {
                let body_bytes = to_bytes(req.body_mut()).await;
                match body_bytes {
                    Ok(bytes) => match TextData::from_bytes(bytes.as_ref().to_vec()) {
                        Ok(text) => RequestBody::new_text(text),
                        Err(_) => RequestBody::new_binary(bytes),
                    },
                    Err(err) => return Err(err),
                }
            }
            RequestBodyType::BINARY => {
                let body_bytes = to_bytes(req.body_mut()).await;

                match body_bytes {
                    Ok(bytes) => RequestBody::new_binary(bytes),
                    Err(err) => return Err(err),
                }
            }
            RequestBodyType::EMPTY => RequestBody {
                content: RequestBodyContent::EMPTY,
                content_type: RequestBodyType::EMPTY,
            },
        };

        let is_secure = protocol == String::from("https");
        let xhr_header = headers.get("X-Requested-With").unwrap_or("");
        let xhr = xhr_header == "XMLHttpRequest";

        Ok(HttpRequest {
            params,
            query: query,
            origin_url,
            method,
            ip,
            path,
            protocol,
            headers,
            data,
            body: request_body,
            cookies: cookies_map,
            xhr,
            is_secure,
        })
    }

    pub(crate) fn from_request_info(req_info: RequestInfo) -> Self {
        let mut headers = RequestHeaders::new();

        req_info.headers().iter().for_each(|(key, value)| {
            if let Ok(v) = value.to_str() {
                headers.insert(key.as_str(), v);
            }
        });

        let method = HttpMethods::from(req_info.method());
        let origin_url = match req_info.uri().authority() {
            Some(authority) => {
                let scheme = req_info.uri().scheme_str().unwrap_or("http");
                Url::new(format!("{}://{}", scheme, authority))
            }
            None => {
                let uri_string = req_info
                    .headers()
                    .get(HOST)
                    .and_then(|host| host.to_str().ok())
                    .map(|host| {
                        // Determine scheme (you might want to check for TLS context)
                        let scheme = "http"; // or "https" if using TLS
                        format!("{}://{}", scheme, host)
                    })
                    .unwrap_or(String::new());

                Url::new(uri_string)
            }
        };

        let query_string = req_info.uri().query().unwrap_or("");

        let queries = url::form_urlencoded::parse(query_string.as_bytes())
            .filter_map(|(key, value)| Some((key.to_string(), value.to_string())))
            .collect::<HashMap<String, String>>();

        let query = QueryParams::from_map(queries);
        let mut params = RouteParams::new();

        if let Some(param_routerify) = req_info.data::<routerify::RouteParams>() {
            println!("Params: {:?}", param_routerify);
            param_routerify.iter().for_each(|(key, value)| {
                params.insert(key.to_string(), value.to_string());
            });
        }

        let mut cookies_map = HashMap::new();
        let cookies = Self::get_cookies_from_req_info(&req_info);

        cookies.iter().for_each(|cookie| {
            let (name, value) = (cookie.name(), cookie.value());
            cookies_map.insert(name.to_string(), value.to_string());
        });

        let ip = req_info
            .headers()
            .get("X-Forwarded-For")
            .and_then(|val| val.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
            .unwrap_or(String::new());

        let ip = match ip.parse::<IpAddr>() {
            Ok(ip) => ip,
            Err(_) => IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        };

        let protocol = req_info
            .headers()
            .get("x-forwarded-proto")
            .and_then(|val| val.to_str().ok())
            .unwrap_or("http")
            .to_string();

        let is_secure = protocol == String::from("https");

        let xhr_header = headers.get("x-requested-with").unwrap_or("");
        let xhr = xhr_header == "XMLHttpRequest";

        // Try to get RequestData from the request info
        let mut data = RequestData::new();
        if let Some(ext_data) = req_info.data::<RequestData>() {
            data = ext_data.clone();
        }

        Self {
            body: RequestBody {
                content: RequestBodyContent::EMPTY,
                content_type: RequestBodyType::EMPTY,
            },
            cookies: cookies_map,
            headers,
            is_secure,
            method,
            origin_url,
            params,
            path: req_info.uri().path().to_string(),
            query,
            xhr,
            data,
            ip,
            protocol,
        }
    }

    #[cfg(feature = "with-wynd")]
    pub fn to_hyper_request(&self) -> Result<Request<Body>, Box<dyn std::error::Error>> {
        let path = if self.path.is_empty() {
            "/".to_string()
        } else if !self.path.starts_with('/') {
            return Err("Path must start with '/'".into());
        } else {
            self.path.clone()
        };

        let mut uri_builder = path.to_string();
        if !self.query.is_empty() {
            uri_builder.push('?');
            uri_builder.push_str(&get_all_query(&self.query));
        }

        let uri: hyper::Uri = uri_builder
            .parse()
            .map_err(|e| format!("Failed to parse URI '{}': {}", uri_builder, e))?;

        let mut builder = Request::builder()
            .method(self.method.to_string().as_str())
            .uri(uri);

        // Add headers
        if let Some(headers) = builder.headers_mut() {
            // Add all headers
            for (name, value) in self.headers.iter() {
                if let (Ok(hn), Ok(hv)) = (
                    hyper::header::HeaderName::from_bytes(name.as_bytes()),
                    hyper::header::HeaderValue::from_str(value),
                ) {
                    headers.append(hn, hv);
                }
            }

            if !self.cookies.is_empty() && !headers.contains_key(hyper::header::COOKIE) {
                let cookie_str: String = self
                    .cookies
                    .iter()
                    .map(|(name, value)| format!("{}={}", name, value))
                    .collect::<Vec<_>>()
                    .join("; ");
                headers.insert(
                    hyper::header::COOKIE,
                    hyper::header::HeaderValue::from_str(&cookie_str)?,
                );
            }
        }

        let data = self.get_all_data();

        if let Some(ext) = builder.extensions_mut() {
            ext.insert(data.clone());
        }

        let body = match &self.body.content {
            RequestBodyContent::JSON(json) => {
                builder
                    .headers_mut()
                    .unwrap()
                    .insert(hyper::header::CONTENT_TYPE, "application/json".parse()?);
                Body::from(serde_json::to_string(json)?)
            }
            RequestBodyContent::TEXT(text) => {
                builder
                    .headers_mut()
                    .unwrap()
                    .insert(hyper::header::CONTENT_TYPE, "text/plain".parse()?);
                Body::from(text.as_bytes().to_vec())
            }
            RequestBodyContent::FORM(form) => {
                builder.headers_mut().unwrap().insert(
                    hyper::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded".parse()?,
                );
                Body::from(form.to_string().clone())
            }
            RequestBodyContent::BINARY(bytes) => {
                builder.headers_mut().unwrap().insert(
                    hyper::header::CONTENT_TYPE,
                    "application/octet-stream".parse()?,
                );
                Body::from(bytes.clone())
            }
            RequestBodyContent::BinaryWithFields(bytes, _form_data) => {
                // For multipart forms with files, we send the binary data
                // but the form fields are accessible via form_data()
                builder
                    .headers_mut()
                    .unwrap()
                    .insert(hyper::header::CONTENT_TYPE, "multipart/form-data".parse()?);
                Body::from(bytes.clone())
            }
            RequestBodyContent::EMPTY => Body::empty(),
        };

        Ok(builder.body(body)?)
    }
    #[cfg(not(feature = "with-wynd"))]
    pub(crate) fn to_hyper_request(&self) -> Result<Request<Body>, Box<dyn std::error::Error>> {
        let path = if self.path.is_empty() {
            "/".to_string()
        } else if !self.path.starts_with('/') {
            return Err("Path must start with '/'".into());
        } else {
            self.path.clone()
        };

        let mut uri_builder = path.to_string();
        if !self.query.is_empty() {
            uri_builder.push('?');
            uri_builder.push_str(&get_all_query(&self.query));
        }

        let uri: hyper::Uri = uri_builder
            .parse()
            .map_err(|e| format!("Failed to parse URI '{}': {}", uri_builder, e))?;

        let mut builder = Request::builder()
            .method(self.method.to_string().as_str())
            .uri(uri);

        // Add headers
        if let Some(headers) = builder.headers_mut() {
            // Add all headers
            for (name, value) in self.headers.iter() {
                if let (Ok(hn), Ok(hv)) = (
                    hyper::header::HeaderName::from_bytes(name.as_bytes()),
                    hyper::header::HeaderValue::from_str(value),
                ) {
                    headers.append(hn, hv);
                }
            }

            if !self.cookies.is_empty() && !headers.contains_key(hyper::header::COOKIE) {
                let cookie_str: String = self
                    .cookies
                    .iter()
                    .map(|(name, value)| format!("{}={}", name, value))
                    .collect::<Vec<_>>()
                    .join("; ");
                headers.insert(
                    hyper::header::COOKIE,
                    hyper::header::HeaderValue::from_str(&cookie_str)?,
                );
            }
        }

        let data = self.get_all_data();

        if let Some(ext) = builder.extensions_mut() {
            ext.insert(data.clone());
        }

        let body = match &self.body.content {
            RequestBodyContent::JSON(json) => {
                builder
                    .headers_mut()
                    .unwrap()
                    .insert(hyper::header::CONTENT_TYPE, "application/json".parse()?);
                Body::from(serde_json::to_string(json)?)
            }
            RequestBodyContent::TEXT(text) => {
                builder
                    .headers_mut()
                    .unwrap()
                    .insert(hyper::header::CONTENT_TYPE, "text/plain".parse()?);
                Body::from(text.as_bytes().to_vec())
            }
            RequestBodyContent::FORM(form) => {
                builder.headers_mut().unwrap().insert(
                    hyper::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded".parse()?,
                );
                Body::from(form.to_string().clone())
            }
            RequestBodyContent::BINARY(bytes) => {
                builder.headers_mut().unwrap().insert(
                    hyper::header::CONTENT_TYPE,
                    "application/octet-stream".parse()?,
                );
                Body::from(bytes.clone())
            }
            RequestBodyContent::BinaryWithFields(bytes, _form_data) => {
                // For multipart forms with files, we send the binary data
                // but the form fields are accessible via form_data()
                builder
                    .headers_mut()
                    .unwrap()
                    .insert(hyper::header::CONTENT_TYPE, "multipart/form-data".parse()?);
                Body::from(bytes.clone())
            }
            RequestBodyContent::EMPTY => Body::empty(),
        };

        Ok(builder.body(body)?)
    }
}

#[cfg(test)]
impl HttpRequest {
    pub(crate) fn set_query(&mut self, key: &str, value: &str) {
        self.query.insert(key.to_string(), value.to_string());
    }

    pub(crate) fn set_header(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }

    pub(crate) fn set_cookie(&mut self, key: &str, value: &str) {
        self.cookies.insert(key.to_string(), value.to_string());
    }

    pub(crate) fn set_json<J>(&mut self, json: J, content_type: RequestBodyType)
    where
        J: serde::de::DeserializeOwned + serde::Serialize,
    {
        self.body.content_type = content_type;
        self.body.content = RequestBodyContent::JSON(serde_json::to_value(json).unwrap());
    }

    pub(crate) fn set_text(&mut self, text: TextData, content_type: RequestBodyType) {
        self.body.content_type = content_type;
        self.body.content = RequestBodyContent::TEXT(text)
    }

    pub(crate) fn set_form(
        &mut self,
        key: &'static str,
        value: &'static str,
        content_type: RequestBodyType,
    ) {
        self.body.content_type = content_type;

        match &mut self.body.content {
            RequestBodyContent::FORM(existing) => {
                existing.insert(key, value);
            }
            _ => {
                let mut form_data = FormData::new();
                form_data.insert(key, value);

                self.body.content = RequestBodyContent::FORM(form_data)
            }
        }
    }

    pub(crate) fn set_content_type(&mut self, content_type: RequestBodyType) {
        self.body.content_type = content_type;
    }

    pub(crate) fn set_binary(&mut self, bytes: Vec<u8>) {
        self.body.content_type = RequestBodyType::BINARY;
        self.body.content = RequestBodyContent::BINARY(bytes.into());
    }

    pub(crate) fn set_method(&mut self, method: HttpMethods) {
        self.method = method;
    }

    pub(crate) fn set_path(&mut self, path: String) {
        self.path = path;
    }

    pub(crate) fn set_origin_url(&mut self, origin_url: Url) {
        self.origin_url = origin_url;
    }
}

pub(crate) fn determine_content_type_request(content_type: &str) -> RequestBodyType {
    match content_type.parse::<Mime>() {
        Ok(mime_type) => match (mime_type.type_(), mime_type.subtype()) {
            (mime::APPLICATION, mime::JSON) => RequestBodyType::JSON,
            (mime::APPLICATION, subtype) if subtype == "x-www-form-urlencoded" => {
                RequestBodyType::FORM
            }
            // Remove the incorrect line that was matching application/form-data as multipart
            (mime::MULTIPART, subtype) if subtype == "form-data" => RequestBodyType::MultipartForm,
            (mime::TEXT, _) => RequestBodyType::TEXT,
            // Handle JSON variants
            (mime::APPLICATION, subtype) if subtype.as_str().ends_with("+json") => {
                RequestBodyType::JSON
            }
            (mime::APPLICATION, subtype)
                if subtype == "xml" || subtype.as_str().ends_with("+xml") =>
            {
                RequestBodyType::TEXT
            }
            _ => RequestBodyType::BINARY,
        },
        Err(_) => RequestBodyType::BINARY, // Fallback for invalid MIME types
    }
}

pub(crate) fn determine_content_type_response(content_type: &str) -> ResponseContentType {
    match content_type.parse::<Mime>() {
        Ok(mime_type) => match (mime_type.type_(), mime_type.subtype()) {
            (mime::APPLICATION, mime::JSON) => ResponseContentType::JSON,
            (mime::TEXT, _) => ResponseContentType::TEXT,
            (mime::APPLICATION, subtype) if subtype.as_str().ends_with("+json") => {
                ResponseContentType::JSON
            }
            (mime::APPLICATION, subtype)
                if subtype == "xml" || subtype.as_str().ends_with("+xml") =>
            {
                ResponseContentType::TEXT
            }
            _ => ResponseContentType::BINARY,
        },
        Err(_) => ResponseContentType::BINARY, // Fallback for invalid MIME types
    }
}

#[cfg(feature = "with-wynd")]
impl AsyncWrite for HttpRequest {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        // Get a mutable reference to self
        let this = self.get_mut();

        // Convert the buffer to bytes
        let new_bytes = bytes::Bytes::copy_from_slice(buf);

        // Append the new bytes to the existing body content
        match &mut this.body.content {
            RequestBodyContent::BINARY(existing_bytes) => {
                // For binary content, we need to concatenate the bytes
                // Since Bytes doesn't support direct concatenation, we'll convert to Vec and back
                let mut combined = existing_bytes.to_vec();
                combined.extend_from_slice(buf);
                this.body.content = RequestBodyContent::BINARY(combined.into());
            }
            RequestBodyContent::BinaryWithFields(existing_bytes, form_data) => {
                // For binary with fields, append to the binary part
                let mut combined = existing_bytes.to_vec();
                combined.extend_from_slice(buf);
                this.body.content =
                    RequestBodyContent::BinaryWithFields(combined.into(), form_data.clone());
            }
            RequestBodyContent::TEXT(text_data) => {
                // For text content, append the bytes as UTF-8 string
                if let Ok(new_text) = String::from_utf8(buf.to_vec()) {
                    // Use as_str_lossy() to handle potential UTF-8 errors gracefully
                    let existing_text = text_data.as_str_lossy();
                    let combined_text = format!("{}{}", existing_text, new_text);
                    this.body.content = RequestBodyContent::TEXT(TextData::new(combined_text));
                } else {
                    // If the new bytes aren't valid UTF-8, convert to binary
                    let mut combined = text_data.as_bytes().to_vec();
                    combined.extend_from_slice(buf);
                    this.body.content = RequestBodyContent::BINARY(combined.into());
                    this.body.content_type = RequestBodyType::BINARY;
                }
            }
            RequestBodyContent::JSON(json_value) => {
                // For JSON content, append as text (this might not be valid JSON)
                let json_str = json_value.to_string();
                let mut combined = json_str.as_bytes().to_vec();
                combined.extend_from_slice(buf);
                // Convert to text since the result might not be valid JSON
                if let Ok(combined_text) = String::from_utf8(combined.clone()) {
                    this.body.content = RequestBodyContent::TEXT(TextData::new(combined_text));
                    this.body.content_type = RequestBodyType::TEXT;
                } else {
                    this.body.content = RequestBodyContent::BINARY(combined.into());
                    this.body.content_type = RequestBodyType::BINARY;
                }
            }
            RequestBodyContent::FORM(form_data) => {
                // For form data, append as text
                let form_str = form_data.to_string();
                let mut combined = form_str.as_bytes().to_vec();
                combined.extend_from_slice(buf);
                if let Ok(combined_text) = String::from_utf8(combined.clone()) {
                    this.body.content = RequestBodyContent::TEXT(TextData::new(combined_text));
                    this.body.content_type = RequestBodyType::TEXT;
                } else {
                    this.body.content = RequestBodyContent::BINARY(combined.into());
                    this.body.content_type = RequestBodyType::BINARY;
                }
            }
            RequestBodyContent::EMPTY => {
                // For empty content, start with the new bytes
                this.body.content = RequestBodyContent::BINARY(new_bytes);
                this.body.content_type = RequestBodyType::BINARY;
            }
        }

        std::task::Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        // No buffering to flush, so we're always ready
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        // No special shutdown needed for request body
        std::task::Poll::Ready(Ok(()))
    }
}

#[cfg(feature = "with-wynd")]
impl AsyncRead for HttpRequest {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        // Get a mutable reference to self
        let this = unsafe { self.get_unchecked_mut() };

        // Convert the request body content to bytes
        let body_bytes = match &this.body.content {
            RequestBodyContent::TEXT(text_data) => text_data.as_bytes().to_vec(),
            RequestBodyContent::JSON(json_value) => {
                serde_json::to_vec(json_value).unwrap_or_default()
            }
            RequestBodyContent::FORM(form_data) => form_data.to_string().as_bytes().to_vec(),
            RequestBodyContent::BINARY(bytes) => bytes.to_vec(),
            RequestBodyContent::BinaryWithFields(bytes, _form_data) => bytes.to_vec(),
            RequestBodyContent::EMPTY => Vec::new(),
        };

        // If we have data to read
        if !body_bytes.is_empty() {
            let bytes_to_copy = std::cmp::min(buf.remaining(), body_bytes.len());
            let start_pos = 0;
            let end_pos = bytes_to_copy;

            // Copy bytes to the buffer
            buf.put_slice(&body_bytes[start_pos..end_pos]);

            // Remove the bytes we just read from the body
            if bytes_to_copy == body_bytes.len() {
                // If we read all bytes, set to empty and sync content_type
                this.body.content = RequestBodyContent::EMPTY;
                this.body.content_type = RequestBodyType::EMPTY;
            } else {
                // If we read partial bytes, update the body with remaining bytes
                let remaining_bytes = body_bytes[end_pos..].to_vec();
                match &this.body.content {
                    RequestBodyContent::TEXT(_) => {
                        if let Ok(remaining_text) = String::from_utf8(remaining_bytes.clone()) {
                            this.body.content =
                                RequestBodyContent::TEXT(TextData::new(remaining_text));
                        } else {
                            this.body.content =
                                RequestBodyContent::BINARY(remaining_bytes.clone().into());
                            this.body.content_type = RequestBodyType::BINARY;
                        }
                    }
                    RequestBodyContent::JSON(_) => {
                        // For JSON, convert remaining bytes to text or binary
                        if let Ok(remaining_text) = String::from_utf8(remaining_bytes.clone()) {
                            this.body.content =
                                RequestBodyContent::TEXT(TextData::new(remaining_text));
                            this.body.content_type = RequestBodyType::TEXT;
                        } else {
                            this.body.content =
                                RequestBodyContent::BINARY(remaining_bytes.clone().into());
                            this.body.content_type = RequestBodyType::BINARY;
                        }
                    }
                    RequestBodyContent::FORM(_) => {
                        // For form data, convert remaining bytes to text or binary
                        if let Ok(remaining_text) = String::from_utf8(remaining_bytes.clone()) {
                            this.body.content =
                                RequestBodyContent::TEXT(TextData::new(remaining_text));
                            this.body.content_type = RequestBodyType::TEXT;
                        } else {
                            this.body.content =
                                RequestBodyContent::BINARY(remaining_bytes.clone().into());
                            this.body.content_type = RequestBodyType::BINARY;
                        }
                    }
                    RequestBodyContent::BINARY(_) => {
                        this.body.content = RequestBodyContent::BINARY(remaining_bytes.into());
                    }
                    RequestBodyContent::BinaryWithFields(_, form_data) => {
                        this.body.content = RequestBodyContent::BinaryWithFields(
                            remaining_bytes.into(),
                            form_data.clone(),
                        );
                    }
                    RequestBodyContent::EMPTY => {
                        // Should not happen, but handle gracefully
                    }
                }
            }
        }

        // Always return Ready since we're reading from memory
        std::task::Poll::Ready(Ok(()))
    }
}
