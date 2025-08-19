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
            // Read the raw body to determine Content-Type and extract fields/files
            let bytes_vec = match req.bytes() {
                Ok(bytes) => bytes.to_vec(),
                Err(_) => {
                    // If bytes() fails, try to get the body content in other ways
                    // For multipart forms, we need to check the content type first
                    let content_type = req.headers.content_type().unwrap_or_default();
                    if content_type.to_lowercase().contains("multipart/form-data") {
                        // For multipart forms, we need the raw body
                        // Since we can't access it directly, we'll return early
                        // This is a limitation of the current architecture
                        eprintln!(
                            "File upload middleware: multipart/form-data detected but raw body not accessible"
                        );
                        return (req, None);
                    } else {
                        // Not a multipart form, continue without file upload
                        return (req, None);
                    }
                }
            };

            // Determine Content-Type and extract boundary (case-insensitive)
            let content_type = req.headers.content_type().unwrap_or_default();
            let boundary = if content_type.to_lowercase().contains("multipart/form-data") {
                extract_boundary(&content_type)
            } else {
                None
            };

            // Parse multipart/form-data
            let (fields, file_parts) = if let Some(boundary) = boundary {
                parse_multipart_form(&bytes_vec, &boundary)
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
            let mut first_file_info: Option<(String, String)> = None;

            // Process all files
            for (file_bytes, field_name_opt) in files_to_process {
                let (file_bytes, original_filename, field_name) = match field_name_opt {
                    Some(field) => {
                        // If field_name_opt is Some, try to split into original_filename and field_name
                        // If the tuple is (Vec<u8>, Some("filename")), treat as (file_bytes, None, Some("filename"))
                        (file_bytes, None, Some(field))
                    }
                    None => (file_bytes, None, None),
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

                        // Store file info
                        let file_info = FileInfo {
                            filename: filename.clone(),
                            path: filename_with_path.clone(),
                            original_filename: original_filename.clone(),
                            _field_name: field_name.clone(),
                        };
                        uploaded_files.push(file_info);

                        // SIMPLIFIED: Only map the form field name to the UUID filename
                        if let Some(ref field_name) = field_name {
                            req.insert_form_field(field_name, &filename);
                        }

                        // Keep track of first file for backwards compatibility
                        if first_file_info.is_none() {
                            first_file_info = Some((filename, filename_with_path));
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to create file '{}': {}", filename_with_path, e);
                        continue; // Skip this file but continue with others
                    }
                }
            }

            // Set minimal request data for uploaded files
            if !uploaded_files.is_empty() {
                // Set count in data (not form fields)
                req.set_data("uploaded_file_count", &uploaded_files.len().to_string());

                // Create JSON representation of all files in data (not form fields)
                let files_json = format!(
                    "[{}]",
                    uploaded_files
                        .iter()
                        .map(|f| format!(
                            r#"{{"filename":"{}","path":"{}","original_filename":{}}}"#,
                            f.filename,
                            f.path,
                            f.original_filename
                                .as_ref()
                                .map(|s| format!(r#""{}""#, s))
                                .unwrap_or_else(|| "null".to_string())
                        ))
                        .collect::<Vec<_>>()
                        .join(",")
                );
                req.set_data("uploaded_files", &files_json);

                // Backwards compatibility: set data for first file (not form fields)
                if let Some((first_filename, first_path)) = first_file_info {
                    req.set_data("uploaded_file", &first_filename);
                    req.set_data("uploaded_file_path", &first_path);
                }

                // Set original filename for first file if available (in data, not form fields)
                if let Some(orig) = &uploaded_files[0].original_filename {
                    req.set_data("original_filename", orig);
                }
            }

            (req, None)
        })
    }
}

#[derive(Debug)]
struct FileInfo {
    filename: String,
    path: String,
    original_filename: Option<String>,
    _field_name: Option<String>,
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

// Updated multipart parser that extracts text fields and ALL file parts
// Returns (fields, file_parts) where file_parts is Vec<(bytes, field_name)>
fn parse_multipart_form(
    body: &[u8],
    boundary: &str,
) -> (Vec<(String, String)>, Vec<(Vec<u8>, Option<String>)>) {
    let boundary_start = format!("--{}", boundary);
    let boundary_start_bytes = boundary_start.as_bytes();
    let boundary_next = format!("\r\n--{}", boundary);
    let boundary_next_bytes = boundary_next.as_bytes();
    let boundary_close = format!("--{}--", boundary);
    let boundary_close_bytes = boundary_close.as_bytes();

    // Find the first boundary
    let mut pos = match find_subsequence(body, boundary_start_bytes) {
        Some(p) => p + boundary_start_bytes.len(),
        None => return (Vec::new(), Vec::new()),
    };

    // Skip optional CRLF after the first boundary
    if body.get(pos..pos + 2) == Some(b"\r\n") {
        pos += 2;
    }

    let mut fields: Vec<(String, String)> = Vec::new();
    let mut file_parts: Vec<(Vec<u8>, Option<String>)> = Vec::new();

    loop {
        // Find end of headers (CRLFCRLF)
        let header_end_rel = match find_subsequence(&body[pos..], b"\r\n\r\n") {
            Some(i) => i,
            None => return (fields, file_parts),
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
                    None => return (fields, file_parts),
                }
            }
        };
        let content_end = content_start + next_boundary_rel;

        // Parse Content-Disposition to determine field name and if this is a file part
        let mut is_file_part = false;
        let mut field_name: Option<String> = None;
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
                    }
                }
            }
        }

        if is_file_part {
            // Collect ALL file parts, not just the first one
            let file_bytes = trim_trailing_crlf(&body[content_start..content_end]).to_vec();
            // Pass the field_name instead of original_filename for the mapping
            file_parts.push((file_bytes, field_name));
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
            return (fields, file_parts);
        } else {
            // Try to realign to the next boundary start
            match find_subsequence(&body[pos..], boundary_next_bytes) {
                Some(rel) => pos += rel + boundary_next_bytes.len(),
                None => return (fields, file_parts),
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
