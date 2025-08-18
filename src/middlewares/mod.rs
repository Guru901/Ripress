/// # Middlewares

/// This module provides Cross-Origin Resource Sharing (CORS) middleware for handling
/// CORS headers and requests in your application. Use this middleware to control
/// which origins are allowed to access your resources, set allowed methods, headers,
/// and handle preflight requests automatically.
pub mod cors;

/// Logger middleware
///
/// This module provides middleware for logging HTTP requests and responses.
/// It can be used to log details such as the request method, path and response
/// time for debugging and monitoring purposes.
pub mod logger;

/// File uploader middleware
///
/// This module provides middleware for handling file uploads in your application.
/// It processes binary request bodies and saves files to a configurable upload directory
/// with automatic extension detection and unique filename generation.
///
/// ## Features
///
/// * **Binary file processing** - Handles raw binary content uploads
/// * **Automatic extension detection** - Uses the `infer` crate to detect file types
/// * **Unique filenames** - Generates UUID-based names to prevent conflicts
/// * **Configurable storage** - Customizable upload directory with fallback
/// * **Non-blocking operation** - Continues request processing even if uploads fail
/// * **Error logging** - Comprehensive logging for debugging upload issues
///
/// ## Usage
///
/// ```rust
/// use ripress::{app::App, middlewares::file_upload::file_upload};
///
/// let mut app = App::new();
/// app.use_middleware("/upload", file_upload(Some("uploads")));
/// ```
///
/// ## Limitations
///
/// * Saves only the first file from `multipart/form-data` requests
/// * Parses text fields from multipart bodies and merges them into `req.form_data()`
pub mod file_upload;
