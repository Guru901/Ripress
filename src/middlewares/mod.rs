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
/// * **Multipart form support** - Extracts ALL file parts and text fields from `multipart/form-data`
/// * **Automatic extension detection** - Uses the `infer` crate to detect file types
/// * **Unique filenames** - Generates UUID-based names to prevent conflicts
/// * **Configurable storage** - Customizable upload directory with fallback
/// * **Non-blocking operation** - Continues request processing even if uploads fail
/// * **Error logging** - Comprehensive logging for debugging upload issues
/// * **Form field mapping** - Maps form field names to generated UUID filenames
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
/// ## Request Data Added
///
/// When files are successfully uploaded, the middleware adds these fields to the request:
///
/// * `uploaded_file_count` - Number of files successfully uploaded
/// * `uploaded_files` - JSON array of uploaded file info (filenames, paths, original names)
/// * For backwards compatibility (first file only):
///   * `uploaded_file` - The generated filename of the first file
///   * `uploaded_file_path` - The full path where the first file was saved
///   * `original_filename` - Original filename if available
///
/// ## Form Field Access
///
/// For multipart forms, text fields are automatically extracted and available via `req.form_data()`.
/// File field names are mapped to their generated UUID filenames for easy access.
///
/// ## Limitations
///
/// * Works with `RequestBodyType::BINARY` content
/// * For `multipart/form-data`, ALL file parts are extracted and saved
/// * Files are saved with UUID-based names to prevent conflicts
/// * The middleware automatically handles directory creation
/// * Upload failures are logged to stderr for debugging
pub mod file_upload;
