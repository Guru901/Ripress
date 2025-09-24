#![warn(missing_docs)]

//! # Ripress
//!
//! Ripress is a lightweight, modular web framework for building HTTP APIs and web applications in Rust.
//! It provides a simple and flexible API for defining routes, handling requests and responses, and composing middleware.
//! Inspired by Express.js, Ripress brings the familiar developer experience to Rust while maintaining high performance.
//!
//! ## Quick Start
//!
//! ```no_run
//! use ripress::{app::App, types::RouterFns};
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut app = App::new();
//!
//!     // Define routes
//!     app.get("/", |_req, res| async move {
//!         res.ok().text("Hello, World!")
//!     });
//!
//!     app.get("/api/users", |_req, res| async move {
//!         res.ok().json(serde_json::json!({
//!             "users": ["Alice", "Bob", "Charlie"]
//!         }))
//!     });
//!
//!     // Add middleware
//!     app.use_cors(None)
//!         .use_logger(None);
//!
//!     // Start server
//!     app.listen(3000, || {
//!         println!("Server running on http://localhost:3000");
//!     }).await;
//! }
//! ```
//!
//! ## Key Features
//!
//! - **Express.js-like API**: Familiar routing and middleware patterns
//! - **Async/Await Support**: Built on Tokio for high-performance async operations
//! - **Type Safety**: Full Rust type safety with compile-time error checking
//! - **Built-in Middleware**: CORS, logging, compression, rate limiting, and more
//! - **Request/Response Objects**: Rich APIs for handling HTTP data
//! - **WebSocket Support**: Real-time communication via the `wynd` crate
//! - **Static File Serving**: Built-in support for serving static assets
//!
//! ## Advanced Examples
//!
//! ### RESTful API with JSON
//! ```no_run
//! use ripress::{app::App, types::RouterFns};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct User {
//!     id: u32,
//!     name: String,
//!     email: String,
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut app = App::new();
//!
//!     // GET /users - List all users
//!     app.get("/users", |_req, res| async move {
//!         let users = vec![
//!             User { id: 1, name: "Alice".to_string(), email: "alice@example.com".to_string() },
//!             User { id: 2, name: "Bob".to_string(), email: "bob@example.com".to_string() },
//!         ];
//!         res.ok().json(users)
//!     });
//!
//!     // POST /users - Create a new user
//!     app.post("/users", |req, res| async move {
//!         match req.json::<User>() {
//!             Ok(user) => res.created().json(user),
//!             Err(_) => res.bad_request().text("Invalid JSON"),
//!         }
//!     });
//!
//!     // GET /users/:id - Get user by ID
//!     app.get("/users/:id", |req, res| async move {
//!         let user_id = req.params.get("id").unwrap_or("0");
//!         res.ok().json(serde_json::json!({
//!             "id": user_id,
//!             "message": "User found"
//!         }))
//!     });
//!
//!     app.listen(3000, || {
//!         println!("API server running on http://localhost:3000");
//!     }).await;
//! }
//! ```
//!
//! ### File Upload with Middleware
//! ```no_run
//! use ripress::{app::App, middlewares::file_upload::file_upload, types::RouterFns};
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut app = App::new();
//!
//!     // Add file upload middleware
//!     app.use_pre_middleware("/upload", file_upload(None));
//!
//!     app.post("/upload", |req, res| async move {
//!         // Access uploaded files through request data
//!         if let Some(file_data) = req.get_data("uploaded_file") {
//!             res.ok().text(format!("File uploaded: {}", file_data))
//!         } else {
//!             res.bad_request().text("No file uploaded")
//!         }
//!     });
//!
//!     app.listen(3000, || {
//!         println!("File upload server running on http://localhost:3000");
//!     }).await;
//! }
//! ```
//!

/// The main application struct and its methods for configuring and running your server.
///
/// The `App` struct is the core of Ripress, providing methods to define routes, add middleware,
/// and start the HTTP server. It follows an Express.js-like pattern for route handling.
///
/// # Examples
///
/// Basic server setup:
/// ```rust
/// use ripress::{app::App, types::RouterFns};
///
/// #[tokio::main]
/// async fn main() {
///     let mut app = App::new();
///     app.get("/", |_req, res| async move { res.ok().text("Hello, World!") } );
/// }
/// ```
///
/// With middleware:
/// ```rust
/// use ripress::{app::App, types::RouterFns};
///
/// #[tokio::main]
/// async fn main() {
///     let mut app = App::new();
///
///     app.use_cors(None)
///         .use_logger(None)
///         .get("/api/data", |_req, res| async move {
///             res.ok().json(serde_json::json!({"status": "ok"}))
///         });
/// }
/// ```
pub mod app;

/// The HTTP request struct and its methods for extracting data from requests.
///
/// `HttpRequest` provides comprehensive access to incoming HTTP request data including
/// headers, cookies, query parameters, route parameters, and request body content.
///
/// # Examples
///
/// Accessing request data:
/// ```rust
/// use ripress::context::HttpRequest;
///
/// async fn handler(req: HttpRequest, res: ripress::context::HttpResponse) -> ripress::context::HttpResponse {
///     // Get query parameters
///     let name = req.query.get("name").unwrap_or("World");
///     
///     // Get route parameters
///     let user_id = req.params.get("id");
///     
///     // Parse JSON body
///     if let Ok(data) = req.json::<serde_json::Value>() {
///         println!("Received JSON: {:?}", data);
///     }
///     
///     res.ok().text(format!("Hello, {}!", name))
/// }
/// ```
///
/// See [`req::HttpRequest`] for details.
pub mod req;

/// The HTTP response struct and its methods for building responses.
///
/// `HttpResponse` provides methods to construct HTTP responses with different status codes,
/// headers, cookies, and various body types (JSON, text, HTML, binary).
///
/// # Examples
///
/// Creating responses:
/// ```rust
/// use ripress::context::{HttpResponse, HttpRequest};
///
/// // JSON response
/// async fn json_res(req: HttpRequest, res: HttpResponse) -> HttpResponse {
///     return res.ok().json(serde_json::json!({"message": "Success"}));
/// }
///
/// // Text response with custom status
/// async fn text_res(req: HttpRequest, res: HttpResponse) -> HttpResponse {
///     return res.status(201).text("Resource created");
/// }
///
/// // Response with cookies
/// async fn cookie_res(req: HttpRequest, res: HttpResponse) -> HttpResponse {
///     return res.ok().set_cookie("session", "abc123", None).text("Logged in");
/// }
/// ```
///
/// See [`res::HttpResponse`] for details.
pub mod res;

/// Common context types for handler functions.
///
/// Re-exports [`HttpRequest`] and [`HttpResponse`] for convenience in route handlers.
/// This module provides the most commonly used types when writing route handlers.
///
/// # Examples
///
/// ```rust
/// use ripress::context::{HttpRequest, HttpResponse};
///
/// async fn my_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
///     res.ok().text("Hello from handler!")
/// }
/// ```
pub mod context {
    pub use super::req::HttpRequest;
    pub use super::res::HttpResponse;
}

/// Utility functions and helpers for common web tasks.
///
/// This module contains helper functions for common web development tasks such as
/// parsing multipart forms, handling query parameters, and other utilities.
pub mod helpers;

/// Built-in middleware modules for CORS, logging, file uploads, and rate limiting.
///
/// Ripress includes several built-in middleware modules to handle common web application
/// concerns. These can be easily added to your application using the `App` methods.
///
/// # Available Middleware
///
/// - **CORS**: Cross-Origin Resource Sharing configuration
/// - **Logger**: Request/response logging with customizable output
/// - **Compression**: Response compression (gzip, deflate)
/// - **Rate Limiter**: Request rate limiting and throttling
/// - **Body Limit**: Request body size limiting
/// - **Shield**: Security headers and protection
/// - **File Upload**: Multipart form data and file upload handling
///
/// # Examples
///
/// ```rust
/// use ripress::{app::App, types::RouterFns};
///
/// #[tokio::main]
/// async fn main() {
///     let mut app = App::new();
///     
///     // Add multiple middleware
///     app.use_cors(None)                    // Enable CORS    
///         .use_logger(None)                 // Enable request logging
///         .use_compression(None)            // Enable response compression
///         .use_rate_limiter(None)           // Enable rate limiting
///         .use_shield(None);                // Add security headers
/// }
/// ```
pub mod middlewares;

/// The router struct and routing logic for organizing endpoints.
///
/// The router module provides functionality for organizing and managing routes
/// in your application. While most applications will use the `App` struct directly,
/// the router can be used for more complex routing scenarios.
pub mod router;

/// Core types, traits, and enums used throughout the framework.
///
/// This module contains the fundamental types, traits, and enums that power
/// the Ripress framework, including HTTP methods, content types, and routing traits.
///
/// # Key Types
///
/// - `HttpMethods`: Enum representing HTTP methods (GET, POST, PUT, etc.)
/// - `RouterFns`: Trait for route definition methods
/// - `ResponseContentType`: Enum for response content types
/// - `RequestBodyType`: Enum for request body types
pub mod types;

/// Internal test module for framework testing.
mod tests;

/// Error types and utilities for the Ripress framework.
///
/// This module provides structured error types, error categories, and conversion utilities
/// for handling errors throughout the framework. It includes the [`RipressError`] struct,
/// the [`RipressErrorKind`] enum for classifying errors, and conversions from lower-level
/// errors such as query parameter and route parameter parsing failures.
///
/// # Error Handling
///
/// Ripress provides comprehensive error handling with structured error types that make it
/// easy to handle different kinds of errors that can occur in web applications.
///
/// # Examples
///
/// Basic error handling:
/// ```rust
/// use ripress::error::{RipressError, RipressErrorKind};
///
/// // Create a custom error
/// let error = RipressError::new(
///     RipressErrorKind::InvalidInput,
///     "Invalid input data".to_string()
/// );
///
/// // Handle different error types
/// match error.kind() {
///     RipressErrorKind::InvalidInput => {
///         println!("Invalid data: {}", error.message());
///     }
///     RipressErrorKind::NotFound => {
///         println!("Resource not found: {}", error.message());
///     }
///     _ => {
///         println!("Other error: {}", error.message());
///     }
/// }
/// ```
///
/// See [`error::RipressError`] and [`error::RipressErrorKind`] for details.
pub mod error;
