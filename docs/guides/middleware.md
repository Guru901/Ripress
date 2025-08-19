# Middleware Documentation

## File Upload Middleware

The file upload middleware handles file uploads by processing request bodies and saving them to a specified upload directory. It supports raw binary uploads and multipart form data with automatic file type detection and unique filename generation.

### Features

- **Automatic file extension detection** using the `infer` crate
- **Unique filename generation** with UUIDs to prevent conflicts
- **Graceful error handling** - continues request processing even if upload fails
- **Configurable upload directory** with fallback to "uploads"
- **Non-blocking operation** - doesn't short-circuit requests on upload failures
- **Supports multipart/form-data** - extracts ALL file parts and text fields
- **Multiple file support** - handles multiple files in a single request

### Usage

```rust
use ripress::{app::App, middlewares::file_upload::file_upload};

// Use default "uploads" directory
let mut app = App::new();
app.use_middleware("/upload", file_upload(None));

// Use custom directory
app.use_middleware("/files", file_upload(Some("user_files")));
```

### How File Processing Works

The middleware processes requests as follows:

1. **Content Detection**: Attempts to read the raw request body
2. **Multipart Parsing**: If Content-Type is `multipart/form-data`, extracts all parts
3. **Text Field Extraction**: Adds text fields to `req.form_data()`
4. **File Processing**: Saves all file content with UUID filenames and detected extensions
5. **Field Mapping**: Maps file input field names to generated UUID filenames in `req.form_data()`
6. **Data Injection**: Sets comprehensive file information in request data
7. **Error Handling**: Logs errors but continues request processing

### Form Field Behavior

**For multipart forms:**

- **Text fields**: Available via `req.form_data()` with original names and values
- **File fields**: Field names are mapped to generated UUID filenames in `req.form_data()`
- **Example**: `<input name="profile_pic" type="file">` → `req.form_data().get("profile_pic")` returns UUID filename like `"a1b2c3d4-e5f6-7890-abcd-ef1234567890.jpg"`

**For single binary uploads:**

- Uses "file" as the default field name
- No original filename preservation

### Request Data Available After Upload

**In req.get_data() (not form_data):**

- `uploaded_file_count` - Number of files successfully uploaded (as string)
- `uploaded_files` - JSON array of file information
- `uploaded_file` - First file's UUID filename (backwards compatibility)
- `uploaded_file_path` - First file's full path (backwards compatibility)
- `original_filename` - First file's original name if available from multipart

**In req.form_data():**

- Text field names → their string values
- File field names → their generated UUID filenames (strings)

### Examples

Processing uploaded files in a route handler:

```rust
async fn upload_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    // Check if files were uploaded
    if let Some(count_str) = req.get_data("uploaded_file_count") {
        let count: usize = count_str.parse().unwrap_or(0);

        if count > 0 {
            // Get detailed file information
            if let Some(files_json) = req.get_data("uploaded_files") {
                res.ok().text(format!("Uploaded {} files: {}", count, files_json))
            } else {
                // Access individual file info (backwards compatibility)
                let filename = req.get_data("uploaded_file").unwrap_or("unknown");
                let path = req.get_data("uploaded_file_path").unwrap_or("unknown");
                res.ok().text(format!("Uploaded file: {} at {}", filename, path))
            }
        } else {
            res.ok().text("No files were uploaded")
        }
    } else {
        res.ok().text("Upload processing not completed")
    }
}

// Accessing form fields (including file field mappings)
async fn form_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    // Get text fields
    if let Some(username) = req.form_data().get("username") {
        println!("Username: {}", username);
    }

    // Get file field mapping (returns UUID filename)
    if let Some(avatar_filename) = req.form_data().get("avatar") {
        println!("Avatar saved as: {}", avatar_filename);
    }

    res.ok().text("Form processed")
}
```

### Current Limitations

1. **Original filename preservation**: Due to tuple handling in the code, original filenames from multipart forms are not properly preserved in individual file processing
2. **Single binary uploads**: Always use "file" as the field name, no original filename
3. **Raw body access**: Requires successful `req.bytes()` call - may fail for some request types
4. **Field mapping**: Only maps field names to UUID filenames; original filename info is not available in form_data

### Error Handling

The middleware is designed to be non-blocking:

- **Upload failures**: Logged to stderr but don't stop request processing
- **Directory creation failures**: Logged but allow the request to continue
- **Body reading failures**: Logged but request continues normally
- **File write failures**: Logged per-file but don't short-circuit the request
- **Multipart parsing errors**: Gracefully fall back to single binary processing

---

## CORS Middleware

The CORS (Cross-Origin Resource Sharing) middleware handles CORS headers and preflight requests to control which origins can access your resources.

### Configuration

The `CorsConfig` struct allows you to customize CORS behavior:

- `allowed_origin` - The allowed origin for requests (default: "\*")
- `allowed_methods` - The allowed HTTP methods (default: "GET, POST, PUT, DELETE, OPTIONS, HEAD")
- `allowed_headers` - The allowed headers (default: "Content-Type, Authorization")
- `allow_credentials` - Whether to allow credentials (default: false)

### Usage

```rust
use ripress::{app::App, middlewares::cors::cors};

// Use default CORS settings
let mut app = App::new();
app.use_middleware("", cors(None));

// Use custom CORS settings
use ripress::middlewares::cors::{cors, CorsConfig};
app.use_middleware("", cors(Some(CorsConfig {
    allowed_origin: "https://example.com",
    allowed_methods: "GET, POST, PUT, DELETE, OPTIONS, HEAD",
    allowed_headers: "Content-Type, Authorization",
    allow_credentials: true,
})));
```

### How It Works

The CORS middleware:

1. **Adds CORS headers** to all responses based on configuration
2. **Handles preflight requests** - automatically responds to OPTIONS requests with a 200 status
3. **Continues processing** for all non-OPTIONS requests after adding headers
4. **Sets credentials header** if `allow_credentials` is true

### Default Configuration

When using `cors(None)`, the middleware applies these defaults:

- **Origin**: `*` (allow all origins)
- **Methods**: `GET, POST, PUT, DELETE, OPTIONS, HEAD`
- **Headers**: `Content-Type, Authorization`
- **Credentials**: `false`

### Headers Added

The middleware automatically adds these headers to responses:

- `Access-Control-Allow-Origin`
- `Access-Control-Allow-Methods`
- `Access-Control-Allow-Headers`
- `Access-Control-Allow-Credentials` (if enabled)

### Preflight Handling

For OPTIONS requests (preflight), the middleware:

- Adds all CORS headers
- Returns a 200 OK response immediately
- Does not continue to other handlers

For all other requests:

- Adds CORS headers
- Continues to the next middleware or route handler

---

## Logger Middleware

The logger middleware logs HTTP request information for debugging and monitoring purposes.

### Configuration

The `LoggerConfig` struct controls what information gets logged:

- `method` - Whether to log the HTTP method (default: true)
- `path` - Whether to log the request path (default: true)
- `duration` - Whether to log the request duration (default: true)

### Usage

```rust
use ripress::{app::App, middlewares::logger::logger};

// Use default logging (logs method, path, and duration)
let mut app = App::new();
app.use_middleware("", logger(None));

// Use custom logging configuration
use ripress::middlewares::logger::{logger, LoggerConfig};
app.use_middleware("", logger(Some(LoggerConfig {
    duration: true,
    method: true,
    path: false, // Don't log the path
})));
```

### How It Works

The logger middleware:

1. **Records start time** when the request begins
2. **Captures request details** (method, path) from the request
3. **Continues processing** - doesn't interrupt the request flow
4. **Calculates duration** after processing
5. **Prints log information** to stdout based on configuration

### Log Format

The logger outputs information in this format:

```
path: /api/users, Time taken: 45ms, method: GET
```

The order and presence of fields depends on your configuration:

- If `path` is true: shows "path: {path}"
- If `duration` is true: shows "Time taken: {ms}ms"
- If `method` is true: shows "method: {method}"

### Default Configuration

When using `logger(None)`, all logging options are enabled:

- **Method**: true
- **Path**: true
- **Duration**: true

### Performance Impact

The logger middleware:

- Uses `std::time::Instant` for precise duration measurement
- Performs minimal string operations
- Does not block request processing
- Outputs synchronously to stdout

### Examples

Different configuration examples:

```rust
// Log everything (default)
app.use_middleware("", logger(None));

// Only log duration and method
app.use_middleware("", logger(Some(LoggerConfig {
    duration: true,
    method: true,
    path: false,
})));

// Only log the request path
app.use_middleware("", logger(Some(LoggerConfig {
    duration: false,
    method: false,
    path: true,
})));

// Disable all logging (not very useful)
app.use_middleware("", logger(Some(LoggerConfig {
    duration: false,
    method: false,
    path: false,
})));
```
