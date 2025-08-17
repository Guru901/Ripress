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
/// This module provides middleware for uploading files to a server.
/// It can be used to upload files to a server and handle file uploads.
pub mod file_upload;
