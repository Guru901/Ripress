#![warn(missing_docs)]
use crate::helpers::{extract_boundary, parse_multipart_form};
use crate::req::body::FormData;
use crate::{context::HttpResponse, req::HttpRequest, types::FutMiddleware};
use tokio::fs::{File, create_dir_all};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

/// Builtin File Upload Middleware
///
/// This middleware handles file uploads by processing request bodies and saving
/// them to a specified upload directory. It supports raw binary uploads
/// (application/octet-stream) and browser uploads via `multipart/form-data`.
/// It automatically generates unique filenames using UUIDs and infers
/// file extensions from the binary content.
///
/// ## Features
///
/// * **Automatic file extension detection** using the `infer` crate
/// * **Unique filename generation** with UUIDs to prevent conflicts
/// * **Graceful error handling** - continues request processing even if upload fails
/// * **Configurable upload directory** with fallback to "uploads"
/// * **File size limits** - configurable maximum file size per file
/// * **File count limits** - configurable maximum number of files per request
/// * **File type filtering** - optional whitelist of allowed file extensions
/// * **Non-blocking operation** - doesn't short-circuit requests on upload failures
/// * **Supports multipart/form-data** - extracts ALL file parts and saves them
/// * **Multiple file support** - handles multiple files in a single request
/// * **Form field preservation** - text fields from multipart forms are preserved
///
/// ## Configuration
///
/// The middleware accepts an optional `FileUploadConfiguration` struct with the following options:
///
/// * `upload_dir` - Directory path for file uploads (default: "uploads")
/// * `max_file_size` - Maximum size per file in bytes (default: 10 MB)
/// * `max_files` - Maximum number of files per request (default: 100)
/// * `allowed_file_types` - Vector of allowed file extensions (default: empty = all types allowed)
///
/// ## Behavior
///
/// The middleware processes requests as follows:
///
/// 1. **Content-Type Detection**: Identifies multipart/form-data vs binary content
/// 2. **Body Extraction**: Retrieves raw bytes from request body with fallback to form data
/// 3. **Multipart Parsing**: If multipart, extracts both file and text fields
/// 4. **Validation**: Checks file count, size, and type limits
/// 5. **Directory Creation**: Automatically creates the upload directory if it doesn't exist
/// 6. **File Processing**: Saves all valid files with unique filenames and detected extensions
/// 7. **Form Field Injection**: Adds text fields to request form data and file field names
/// 8. **Error Handling**: Logs errors but continues request processing without short-circuiting
///
/// ## Form Data Integration
///
/// The middleware integrates with the request's form data system:
///
/// * **Text fields** from multipart forms are added to the request's form data
/// * **File field names** are populated with the generated filenames for uploaded files
/// * **Single binary uploads** use "file" as the default field name
///
/// ## Examples
///
/// Basic usage with default configuration:
///
/// ```rust
/// use ripress::{app::App, middlewares::file_upload::{FileUploadConfiguration, file_upload}};
///
/// let mut app = App::new();
/// app.use_middleware("/upload", file_upload(Some(FileUploadConfiguration::default())));
/// ```
///
/// Custom configuration with size and type limits:
///
/// ```rust
/// use ripress::{app::App, middlewares::file_upload::{file_upload, FileUploadConfiguration}};
///
/// let mut app = App::new();
/// let config = FileUploadConfiguration {
///     upload_dir: "user_uploads".to_string(),
///     max_file_size: 1024 * 1024 * 5, // 5 MB
///     max_files: 10,
///     allowed_file_types: vec!["jpg".to_string(), "png".to_string(), "pdf".to_string()],
/// };
/// app.use_middleware("/upload", file_upload(Some(config)));
/// ```
///
/// Using default configuration (no argument needed):
///
/// ```rust
/// use ripress::{app::App, middlewares::file_upload::file_upload};
///
/// let mut app = App::new();
/// app.use_middleware("/upload", file_upload(None));
/// ```
///
/// Route handler that processes uploaded files:
///
/// ```rust
/// use ripress::context::{HttpRequest, HttpResponse};
///
/// async fn upload_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
///     // Access uploaded file via form field (for single uploads)
///     let form_data = req.form_data().unwrap();
///
///     if let Some(filename) = form_data.get("file") {
///         res.ok().text(format!("File uploaded as: {}", filename))
///     } else {
///         res.ok().text("No file was uploaded")
///     }
/// }
/// ```
///
/// Handling multipart form with multiple files:
///
/// ```rust
/// use ripress::context::{HttpRequest, HttpResponse};
///
/// async fn multi_upload_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
///     let mut uploaded_files = Vec::new();
///     let form_data = req.form_data().unwrap();
///     
///     // Check each possible file field
///     for field_name in ["avatar", "document", "attachment"] {
///         if let Some(filename) = form_data.get(field_name) {
///             uploaded_files.push(format!("{}: {}", field_name, filename));
///         }
///     }
///     
///     // Access text fields from the multipart form
///     let user_name = form_data.get("name").unwrap_or("Anonymous");
///     
///     if uploaded_files.is_empty() {
///         res.ok().text("No files were uploaded")
///     } else {
///         res.ok().text(format!(
///             "User: {}\nUploaded files:\n{}",
///             user_name,
///             uploaded_files.join("\n")
///         ))
///     }
/// }
/// ```
///
/// ## Error Handling
///
/// The middleware is designed to be non-blocking and fault-tolerant:
///
/// * **Upload failures** are logged to stderr but don't stop request processing
/// * **Directory creation failures** are logged but allow the request to continue
/// * **File size exceeded** - individual files are skipped with logging
/// * **Too many files** - entire request is logged but continues without uploads
/// * **Disallowed file types** - individual files are skipped with logging
/// * **File write failures** are logged but don't short-circuit the request
/// * **Body parsing failures** - logged and request continues without uploads
///
/// ## Security Considerations
///
/// * **File type validation** - Use `allowed_file_types` to restrict uploads
/// * **Size limits** - Configure `max_file_size` and `max_files` appropriately
/// * **Unique filenames** - UUID-based names prevent directory traversal and conflicts
/// * **Directory isolation** - Files are saved only within the configured upload directory
/// * **No execution** - Middleware only handles storage, not file execution
///
/// ## Dependencies
///
/// This middleware requires the following crates:
/// * `tokio` - For async file operations and directory creation
/// * `uuid` - For generating unique filenames
/// * `infer` - For detecting file types and extensions
/// * `urlencoding` - For encoding form data (internal use)
///
/// ## Logging
///
/// The middleware logs various events to stderr for debugging:
/// * Directory creation failures
/// * File size limit exceeded
/// * File count limit exceeded  
/// * Disallowed file type attempts
/// * File creation and write failures
/// * Body parsing failures
///
/// ## Performance Notes
///
/// * Files are processed sequentially, not in parallel
/// * Large files are loaded entirely into memory before writing
/// * Directory creation is checked on every request (consider pre-creating directories)
/// * File type detection requires reading file headers
///
/// ## Compatibility
///
/// * Works with both single binary uploads and multipart/form-data
/// * Backwards compatible with existing form field access patterns
/// * Graceful degradation when uploads fail
/// * Case-insensitive Content-Type header detection

/// Configuration struct for the file upload middleware
///
/// This struct defines all configurable aspects of the file upload behavior,
/// including storage location, size limits, and file type restrictions.
#[derive(Clone)]
pub struct FileUploadConfiguration {
    /// Directory path where uploaded files will be stored
    ///
    /// The directory will be created automatically if it doesn't exist.
    /// Relative paths are resolved from the current working directory.
    pub upload_dir: String,

    /// Maximum size allowed for individual files in bytes
    ///
    /// Files exceeding this limit will be skipped and logged.
    /// Default is 10 MB (1024 * 1024 * 10).
    pub max_file_size: u64,

    /// Maximum number of files allowed per request
    ///
    /// Requests with more files will be logged and no files will be uploaded.
    /// Default is 100.
    pub max_files: u64,

    /// List of allowed file extensions (without dots)
    ///
    /// If empty, all file types are allowed. Extensions are detected automatically
    /// using the `infer` crate based on file headers, not filenames.
    /// Example: vec!["jpg".to_string(), "png".to_string(), "pdf".to_string()]
    pub allowed_file_types: Vec<String>,
}

impl Default for FileUploadConfiguration {
    fn default() -> Self {
        Self {
            upload_dir: "uploads".to_string(),
            max_file_size: 1024 * 1024 * 10, // 10 MB
            max_files: 100,
            allowed_file_types: Vec::new(),
        }
    }
}

/// Creates a file upload middleware function
///
/// Returns a middleware function that can be used with `app.use_middleware()` to handle
/// file uploads on specified routes. The middleware processes multipart/form-data and
/// binary uploads, saving files to the configured directory with unique UUID-based names.
///
/// ## Parameters
///
/// * `config` - Optional configuration. If `None`, uses default settings.
///
/// ## Returns
///
/// A middleware function compatible with the ripress framework that:
/// * Processes file uploads from request bodies
/// * Saves files with unique filenames and detected extensions  
/// * Adds form fields for uploaded filenames
/// * Preserves text fields from multipart forms
/// * Handles errors gracefully without blocking requests
///
/// ## Thread Safety
///
/// The returned middleware is `Send + Sync + Clone` and can be safely used
/// across multiple threads and cloned for multiple routes.
pub fn file_upload(
    config: Option<FileUploadConfiguration>,
) -> impl Fn(HttpRequest, HttpResponse) -> FutMiddleware + Send + Sync + Clone + 'static {
    let config = config.unwrap_or_default();
    move |mut req, _res| {
        let config = config.clone();
        let upload_path = config.upload_dir.clone();
        Box::pin(async move {
            // Determine Content-Type and extract boundary (case-insensitive) first
            let content_type = req.headers.content_type().unwrap_or_default();
            let is_multipart = content_type.to_lowercase().contains("multipart/form-data");
            let boundary = if is_multipart {
                extract_boundary(&content_type)
            } else {
                None
            };

            // For multipart forms, we need the raw body bytes
            let bytes_vec = if is_multipart {
                match req.bytes() {
                    Ok(bytes) => bytes.to_vec(),
                    Err(e) => {
                        eprintln!(
                            "File upload middleware: multipart/form-data detected but req.bytes() failed error: {}",
                            e
                        );
                        return (req, None);
                    }
                }
            } else {
                // For non-multipart requests (including binary uploads), try to get bytes
                match req.bytes() {
                    Ok(bytes) => bytes.to_vec(),
                    Err(_) => {
                        // If bytes() fails, try to get form data as fallback
                        match req.form_data() {
                            Ok(form_data) => {
                                let form_string = form_data_to_string(form_data);
                                if form_string.is_empty() {
                                    eprintln!("File upload middleware: No form data available");
                                    return (req, None);
                                }
                                form_string.into_bytes()
                            }
                            Err(_) => {
                                eprintln!(
                                    "File upload middleware: Both bytes() and form_data() failed"
                                );
                                return (req, None);
                            }
                        }
                    }
                }
            };

            // Parse multipart/form-data if applicable
            let (fields, file_parts) = if let Some(ref boundary_str) = boundary {
                parse_multipart_form(&bytes_vec, boundary_str)
            } else {
                (Vec::new(), Vec::new())
            };

            // Insert any text fields into form_data()
            for (k, v) in fields {
                req.insert_form_field(&k, &v);
            }

            // Determine what files to process
            let files_to_process = if !file_parts.is_empty() {
                file_parts
            } else if boundary.is_some() {
                // This was a multipart request but had no file parts
                // Don't create a fallback "file" field
                Vec::new()
            } else {
                // Single binary upload (backwards compatibility) - use "file" as default field name
                vec![(bytes_vec, Some("file".to_string()))]
            };

            if files_to_process.len() > config.max_files as usize {
                eprintln!(
                    "File upload middleware: Too many files ({} > {})",
                    files_to_process.len(),
                    config.max_files
                );
                return (req, None);
            }

            // Ensure the upload directory exists
            if let Err(e) = create_dir_all(&upload_path).await {
                eprintln!("Failed to create upload directory '{}': {}", upload_path, e);
                // Continue without file upload - don't short-circuit the request
                return (req, None);
            }

            let mut uploaded_files = Vec::new();

            // Process all files
            for (file_bytes, field_name_opt) in files_to_process {
                if file_bytes.len() > config.max_file_size as usize {
                    eprintln!(
                        "File upload middleware: File too large ({} bytes > {} bytes)",
                        file_bytes.len(),
                        config.max_file_size
                    );
                    continue;
                }

                let (file_bytes, _original_filename, field_name) = match field_name_opt {
                    Some(field) => {
                        // If field_name_opt is Some, try to split into original_filename and field_name
                        // If the tuple is (Vec<u8>, Some("filename")), treat as (file_bytes, None, Some("filename"))
                        (file_bytes, None::<String>, Some(field))
                    }
                    None => (file_bytes, None::<String>, None),
                };
                let extension = infer::get(&file_bytes)
                    .map(|info| info.extension())
                    .unwrap_or("bin");

                if !config.allowed_file_types.is_empty() {
                    let ext_norm = extension.to_ascii_lowercase();
                    // Accept both "jpg" and "jpeg"
                    let ext_norm = if ext_norm == "jpg" { "jpeg".to_string() } else { ext_norm };
                    let allowed = config
                        .allowed_file_types
                        .iter()
                        .any(|e| {
                            let e = e.to_ascii_lowercase();
                            e == ext_norm || (e == "jpg" && ext_norm == "jpeg")
                        });
                    if !allowed {
                        eprintln!(
                            "File upload middleware: File type '{}' not allowed (allowed types: {:?})",
                            extension, config.allowed_file_types
                        );
                        continue;
                    }
                }

                let id = Uuid::new_v4();
                let filename = format!("{}.{}", id, extension);
                let filename_with_path = format!("{}/{}.{}", upload_path, id, extension);

                match File::create(&filename_with_path).await {
                    Ok(mut file) => {
                        if let Err(e) = file.write_all(&file_bytes).await {
                            eprintln!("Failed to write file '{}': {}", filename_with_path, e);
                            continue; // Skip this file but continue with others
                        }

                        // Track uploaded files for logging
                        uploaded_files.push(filename.clone());

                        // Add the generated filename to the form field
                        if let Some(field_name) = field_name {
                            req.insert_form_field(&field_name, &filename);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to create file '{}': {}", filename_with_path, e);
                        continue; // Skip this file but continue with others
                    }
                }
            }

            (req, None)
        })
    }
}

/// Converts HashMap<String, String> form data to a string representation
///
/// This is an internal helper function used as a fallback when binary data
/// extraction fails. It URL-encodes form key-value pairs into a query string format.
///
/// ## Parameters
///
/// * `form_data` - Reference to the form data HashMap
///
/// ## Returns
///
/// A URL-encoded string representation of the form data, or an empty string
/// if the form data is empty.
fn form_data_to_string(form_data: &FormData) -> String {
    if form_data.is_empty() {
        return String::new();
    }

    form_data
        .iter()
        .map(|(key, value)| {
            format!(
                "{}={}",
                urlencoding::encode(key),
                urlencoding::encode(value)
            )
        })
        .collect::<Vec<_>>()
        .join("&")
}
