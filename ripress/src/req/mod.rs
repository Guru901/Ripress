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
//! use ripress::req::HttpRequest;
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
//!     app.get("/users", |req: HttpRequest, res| async move {
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
//!     app.post("/users", |req: HttpRequest, res| async move {
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
//!     app.get("/users/:id", |req: HttpRequest, res| async move {
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
//! use ripress::req::HttpRequest;
//!
//! let mut app = App::new();
//!
//! app.get("/info", |req: HttpRequest, res| async move {
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
//! use ripress::req::HttpRequest;
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
//! app.get("/dashboard", |req: HttpRequest, res| async move {
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
//! use ripress::req::HttpRequest;
//!
//! let mut app = App::new();
//!
//! app.get("/profile", |req: HttpRequest, res| async move {
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
//! use ripress::req::HttpRequest;
//!
//! let mut app = App::new();
//!
//! app.post("/upload", |req: HttpRequest, res| async move {
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

/// Module providing type conversions from and to hyper structs into the custom structs of this lib.
pub mod conversions;

pub mod request_error;

#[cfg(feature = "with-wynd")]
/// Module providing implementations necessary for using with-wynd feature
pub mod with_wynd;

use crate::{
    req::body::{FormData, RequestBody, RequestBodyContent, RequestBodyType},
    types::HttpMethods,
};
use ahash::AHashMap;
use cookie::Cookie;
use routerify_ng::RequestInfo;
use std::net::{IpAddr, Ipv4Addr};

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
    pub(crate) cookies: AHashMap<String, String>,

    /// The Data set by middleware in the request to be used in the route handler
    pub(crate) data: RequestData,

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
            cookies: AHashMap::new(),
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

    #[doc(hidden)]
    pub fn set_cookie(&mut self, key: &str, value: &str) {
        self.cookies.insert(key.to_string(), value.to_string());
    }
}
