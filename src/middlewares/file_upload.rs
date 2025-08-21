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
/// * **Non-blocking operation** - doesn't short-circuit requests on upload failures
/// * **Supports multipart/form-data** - extracts ALL file parts and saves them
/// * **Multiple file support** - handles multiple files in a single request
///
/// ## Arguments
///
/// * `upload_dir` - Optional directory path for file uploads (defaults to "uploads")
///
/// ## Behavior
///
/// The middleware processes requests as follows:
///
/// 1. **Binary Content Detection**: Processes requests with `RequestBodyType::BINARY`
/// 2. **Directory Creation**: Automatically creates the upload directory if it doesn't exist
/// 3. **File Processing**: Saves all file content with unique filenames and detected extensions
/// 4. **Data Injection**: Sets uploaded file information in request data
/// 5. **Error Handling**: Logs errors but continues request processing without short-circuiting
///
/// ## Request Data Added
///
/// When files are successfully uploaded, the middleware adds these fields to the request:
///
/// * `uploaded_files` - JSON array of uploaded file info (filenames, paths, original names)
/// * `uploaded_file_count` - Number of files successfully uploaded
/// * For backwards compatibility (first file only):
///   * `uploaded_file` - The generated filename of the first file
///   * `uploaded_file_path` - The full path where the first file was saved
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
///     if let Some(count_str) = req.get_data("uploaded_file_count") {
///         let count: usize = count_str.parse().unwrap_or(0);
///         if count > 0 {
///             if let Some(files_json) = req.get_data("uploaded_files") {
///                 res.ok().text(format!("Uploaded {} files: {}", count, files_json))
///             } else {
///                 res.ok().text(format!("Uploaded {} files", count))
///             }
///         } else {
///             res.ok().text("No files were uploaded")
///         }
///     } else {
///         res.ok().text("No files were uploaded")
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
/// * Works with `RequestBodyType::BINARY` content
/// * For `multipart/form-data`, ALL file parts are extracted and saved
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

            // Ensure the upload directory exists
            if let Err(e) = create_dir_all(&upload_path).await {
                eprintln!("Failed to create upload directory '{}': {}", upload_path, e);
                // Continue without file upload - don't short-circuit the request
                return (req, None);
            }

            let mut uploaded_files = Vec::new();

            // Process all files
            for (file_bytes, field_name_opt) in files_to_process {
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
