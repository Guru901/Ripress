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
/// * **Supports multipart/form-data** - extracts the first file part and saves it
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
/// 3. **File Processing**: Saves the file content with a unique filename and detected extension
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
/// * Works with `RequestBodyType::BINARY` content
/// * For `multipart/form-data`, the first file part is extracted and saved
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
                    let bytes_vec = bytes.to_vec();
                    // Detect multipart/form-data and try to extract fields + the first file part
                    let (fields, file_bytes_opt, original_filename_opt) =
                        match req.headers.content_type() {
                            Some(ct) if ct.contains("multipart/form-data") => {
                                // Extract boundary parameter
                                let boundary = extract_boundary(ct);
                                if let Some(boundary) = boundary {
                                    parse_multipart_form(&bytes_vec, &boundary)
                                } else {
                                    (Vec::new(), None, None)
                                }
                            }
                            _ => (Vec::new(), None, None),
                        };

                    // Insert any text fields into form_data()
                    for (k, v) in fields {
                        req.insert_form_field(&k, &v);
                    }

                    let (file_bytes, original_filename) = if let Some(b) = file_bytes_opt {
                        (b, original_filename_opt)
                    } else {
                        (bytes_vec, None)
                    };

                    // Ensure the upload directory exists
                    if let Err(e) = create_dir_all(&upload_path).await {
                        eprintln!("Failed to create upload directory '{}': {}", upload_path, e);
                        // Continue without file upload - don't short-circuit the request
                        return (req, None);
                    }

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
                                // Continue without file upload - don't short-circuit the request
                                return (req, None);
                            }
                            // File upload successful - set the data and also expose via form_data
                            req.set_data("uploaded_file", filename.as_str());
                            req.set_data("uploaded_file_path", filename_with_path.as_str());

                            // Ensure form_data() contains filename and path, so route handlers
                            // can read via req.form_data()
                            req.insert_form_field("filename", &filename);
                            req.insert_form_field("path", &filename_with_path);
                            if let Some(orig) = original_filename {
                                req.set_data("original_filename", orig.as_str());
                                req.insert_form_field("original_filename", &orig);
                            }
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

fn extract_boundary(content_type: &str) -> Option<String> {
    // Example: "multipart/form-data; boundary=----WebKitFormBoundary7MA4YWxkTrZu0gW"
    // or boundary="..."
    for part in content_type.split(';').map(|s| s.trim()) {
        if let Some(rest) = part.strip_prefix("boundary=") {
            let mut b = rest.trim();
            if b.starts_with('"') && b.ends_with('"') && b.len() >= 2 {
                b = &b[1..b.len() - 1];
            }
            if !b.is_empty() {
                return Some(b.to_string());
            }
        }
    }
    None
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

// Very small, forgiving multipart parser that extracts text fields and the first file part (if any)
// Returns (fields, file_bytes, original_filename).
fn parse_multipart_form(
    body: &[u8],
    boundary: &str,
) -> (Vec<(String, String)>, Option<Vec<u8>>, Option<String>) {
    let boundary_start = format!("--{}", boundary);
    let boundary_start_bytes = boundary_start.as_bytes();
    let boundary_next = format!("\r\n--{}", boundary);
    let boundary_next_bytes = boundary_next.as_bytes();
    let boundary_close = format!("--{}--", boundary);
    let boundary_close_bytes = boundary_close.as_bytes();

    // Find the first boundary
    let mut pos = match find_subsequence(body, boundary_start_bytes) {
        Some(p) => p + boundary_start_bytes.len(),
        None => return (Vec::new(), None, None),
    };

    // Skip optional CRLF after the first boundary
    if body.get(pos..pos + 2) == Some(b"\r\n") {
        pos += 2;
    }

    let mut fields: Vec<(String, String)> = Vec::new();
    let mut first_file_bytes: Option<Vec<u8>> = None;
    let mut first_file_original: Option<String> = None;

    loop {
        // Find end of headers (CRLFCRLF)
        let header_end_rel = match find_subsequence(&body[pos..], b"\r\n\r\n") {
            Some(i) => i,
            None => return (fields, first_file_bytes, first_file_original),
        };
        let headers_bytes = &body[pos..pos + header_end_rel];
        let headers_str = String::from_utf8_lossy(headers_bytes);
        let content_start = pos + header_end_rel + 4;

        // Locate the next boundary (start of next part or closing)
        let next_boundary_rel = match find_subsequence(&body[content_start..], boundary_next_bytes)
        {
            Some(i) => i,
            None => {
                // Try close boundary without preceding CRLF (edge case)
                match find_subsequence(&body[content_start..], boundary_close_bytes) {
                    Some(i2) => i2,
                    None => return (fields, first_file_bytes, first_file_original),
                }
            }
        };
        let content_end = content_start + next_boundary_rel;

        // Parse Content-Disposition to determine field name and if this is a file part
        let mut is_file_part = false;
        let mut field_name: Option<String> = None;
        let mut original_filename: Option<String> = None;
        for line in headers_str.lines() {
            let l = line.trim();
            if l.to_ascii_lowercase().starts_with("content-disposition:") {
                // Extract name
                if let Some(idx) = l.find("name=") {
                    let rest = &l[idx + 5..];
                    let name_val = extract_quoted_or_token(rest);
                    if !name_val.is_empty() {
                        field_name = Some(name_val.to_string());
                    }
                }
                // Extract filename if present
                if let Some(idx) = l.find("filename=") {
                    let rest = &l[idx + 9..];
                    let val = extract_quoted_or_token(rest);
                    if !val.is_empty() {
                        is_file_part = true;
                        original_filename = Some(val.to_string());
                    }
                }
            }
        }

        if is_file_part {
            if first_file_bytes.is_none() {
                let file_bytes = trim_trailing_crlf(&body[content_start..content_end]).to_vec();
                first_file_bytes = Some(file_bytes);
                first_file_original = original_filename.clone();
            }
        } else if let Some(name) = field_name {
            let value_bytes = trim_trailing_crlf(&body[content_start..content_end]);
            let value = String::from_utf8_lossy(value_bytes).to_string();
            fields.push((name, value));
        }

        // Move to the next part
        pos = content_end;
        // Step to the boundary marker and beyond
        if body.get(pos..pos + boundary_next_bytes.len()) == Some(boundary_next_bytes) {
            pos += boundary_next_bytes.len();
        } else if body.get(pos..pos + boundary_close_bytes.len()) == Some(boundary_close_bytes) {
            // End reached
            return (fields, first_file_bytes, first_file_original);
        } else {
            // Try to realign to the next boundary start
            match find_subsequence(&body[pos..], boundary_next_bytes) {
                Some(rel) => pos += rel + boundary_next_bytes.len(),
                None => return (fields, first_file_bytes, first_file_original),
            }
        }

        // Skip CRLF after boundary if present
        if body.get(pos..pos + 2) == Some(b"\r\n") {
            pos += 2;
        }
    }
}

fn trim_trailing_crlf(slice: &[u8]) -> &[u8] {
    if slice.ends_with(b"\r\n") {
        &slice[..slice.len() - 2]
    } else {
        slice
    }
}

fn extract_quoted_or_token(input: &str) -> &str {
    let val = input.trim();
    if let Some(start) = val.find('"') {
        let val2 = &val[start + 1..];
        if let Some(end) = val2.find('"') {
            &val2[..end]
        } else {
            val
        }
    } else {
        val.split(';').next().unwrap_or(val).trim()
    }
}
