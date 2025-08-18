use crate::{context::HttpResponse, req::HttpRequest, types::FutMiddleware};
use tokio::fs::{File, create_dir_all};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

/// Builtin File Upload Middleware
///
/// This middleware handles file uploads by processing binary request bodies and saving
/// them to a specified upload directory. It automatically generates unique filenames
/// using UUIDs and infers file extensions from the binary content.
///
/// ## Features
///
/// * **Automatic file extension detection** using the `infer` crate
/// * **Unique filename generation** with UUIDs to prevent conflicts
/// * **Graceful error handling** - continues request processing even if upload fails
/// * **Configurable upload directory** with fallback to "uploads"
/// * **Non-blocking operation** - doesn't short-circuit requests on upload failures
///
/// ## Arguments
///
/// * `upload_dir` - Optional directory path for file uploads (defaults to "uploads")
///
/// ## Behavior
///
/// The middleware processes requests as follows:
///
/// 1. **Binary Content Detection**: Only processes requests with `RequestBodyType::BINARY`
/// 2. **Directory Creation**: Automatically creates the upload directory if it doesn't exist
/// 3. **File Processing**: Saves the binary content with a unique filename and detected extension
/// 4. **Data Injection**: Sets `uploaded_file` and `uploaded_file_path` in request data
/// 5. **Error Handling**: Logs errors but continues request processing without short-circuiting
///
/// ## Request Data Added
///
/// When a file is successfully uploaded, the middleware adds these fields to the request:
///
/// * `uploaded_file` - The generated filename (e.g., "abc123-def456.jpg")
/// * `uploaded_file_path` - The full path where the file was saved
///
/// ## Examples
///
/// Basic usage with default upload directory:
///
/// ```rust
/// use ripress::{app::App, middlewares::file_upload::file_upload};
/// let mut app = App::new();
/// app.use_middleware("/upload", file_upload(None));
/// ```
///
/// Custom upload directory:
///
/// ```rust
/// use ripress::{app::App, middlewares::file_upload::file_upload};
/// let mut app = App::new();
/// app.use_middleware("/upload", file_upload(Some("custom_uploads")));
/// ```
///
/// Route handler that processes uploaded files:
///
/// ```rust
/// use ripress::context::{HttpRequest, HttpResponse};
///
/// async fn upload_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
///     if let Some(filename) = req.get_data("uploaded_file") {
///         let file_path = req.get_data("uploaded_file_path").unwrap_or_default();
///         res.ok().text(format!("File uploaded: {} at {}", filename, file_path))
///     } else {
///         res.ok().text("No file was uploaded")
///     }
/// }
/// ```
///
/// ## Error Handling
///
/// The middleware is designed to be non-blocking:
///
/// * **Upload failures** are logged but don't stop request processing
/// * **Directory creation failures** are logged but allow the request to continue
/// * **Non-binary requests** are logged but processed normally
/// * **File write failures** are logged but don't short-circuit the request
///
/// ## Dependencies
///
/// This middleware requires the following crates:
/// * `tokio` - For async file operations
/// * `uuid` - For generating unique filenames
/// * `infer` - For detecting file types and extensions
///
/// ## Notes
///
/// * Only works with `RequestBodyType::BINARY` content
/// * For multipart/form-data support, consider implementing a separate middleware
/// * Files are saved with UUID-based names to prevent conflicts
/// * The middleware automatically handles directory creation
/// * Upload failures are logged to stderr for debugging
pub fn file_upload(
    upload_dir: Option<&str>,
) -> impl Fn(HttpRequest, HttpResponse) -> FutMiddleware + Send + Sync + Clone + 'static {
    let upload_path = upload_dir.unwrap_or("uploads").to_string();

    move |mut req, _res| {
        let upload_path = upload_path.clone();
        Box::pin(async move {
            // Try to upload file if binary content is available
            match req.bytes() {
                Ok(bytes) => {
                    // Ensure the upload directory exists
                    if let Err(e) = create_dir_all(&upload_path).await {
                        eprintln!("Failed to create upload directory '{}': {}", upload_path, e);
                        // Continue without file upload - don't short-circuit the request
                        return (req, None);
                    }

                    let extension = infer::get(bytes)
                        .map(|info| info.extension())
                        .unwrap_or("bin");

                    let id = Uuid::new_v4();

                    let filename = format!("{}.{}", id, extension);
                    let filename_with_path = format!("{}/{}.{}", upload_path, id, extension);

                    match File::create(&filename_with_path).await {
                        Ok(mut file) => {
                            if let Err(e) = file.write_all(bytes).await {
                                eprintln!("Failed to write file '{}': {}", filename_with_path, e);
                                // Continue without file upload - don't short-circuit the request
                                return (req, None);
                            }
                            // File upload successful - set the data and continue
                            req.set_data("uploaded_file", filename.as_str());
                            req.set_data("uploaded_file_path", filename_with_path.as_str());
                            (req, None)
                        }
                        Err(e) => {
                            eprintln!("Failed to create file '{}': {}", filename_with_path, e);
                            // Continue without file upload - don't short-circuit the request
                            (req, None)
                        }
                    }
                }
                Err(error_msg) => {
                    // Log the error for debugging but don't fail the request
                    eprintln!("File upload middleware: {}", error_msg);

                    // For non-binary requests, just continue without file upload
                    // This allows the request to proceed normally
                    (req, None)
                }
            }
        })
    }
}
