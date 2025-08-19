# File Upload Middleware Documentation Corrections

## Issues Found

### 1. **Multipart Form Field Handling**

**Documentation says:**

> For multipart forms, text fields are automatically extracted and available via `req.form_data()`. File field names are mapped to their generated UUID filenames.

**Code actually does:**

```rust
// Insert any text fields into form_data()
for (k, v) in fields {
    req.insert_form_field(&k, &v);
}

// SIMPLIFIED: Only map the form field name to the UUID filename
if let Some(ref field_name) = field_name {
    req.insert_form_field(field_name, &filename);
}
```

**Issue:** The code maps file field names to UUID filenames in form_data, but the documentation doesn't clearly explain this behavior.

### 2. **File Parts Data Structure**

**Documentation implies:**

> file_parts is Vec<(bytes, field_name)>

**Code actually uses:**

```rust
// Returns (fields, file_parts) where file_parts is Vec<(bytes, field_name)>
fn parse_multipart_form(
    body: &[u8],
    boundary: &str,
) -> (Vec<(String, String)>, Vec<(Vec<u8>, Option<String>)>) {
```

But then in processing:

```rust
let files_to_process = if !file_parts.is_empty() {
    file_parts
} else {
    // Single binary upload (backwards compatibility) - use "file" as default field name
    vec![(bytes_vec, Some("file".to_string()))]
};
```

**Issue:** The tuple structure is inconsistent between multipart parsing and single file handling.

### 3. **Original Filename Handling**

**Code shows:**

```rust
let (file_bytes, original_filename, field_name) = match field_name_opt {
    Some(field) => {
        // If field_name_opt is Some, try to split into original_filename and field_name
        // If the tuple is (Vec<u8>, Some("filename")), treat as (file_bytes, None, Some("filename"))
        (file_bytes, None, Some(field))
    }
    None => (file_bytes, None, None),
};
```

**Issue:** The comment suggests splitting into original_filename and field_name, but the code always sets original_filename to None for this path. The original filename is only captured during multipart parsing but gets lost in the tuple transformation.

### 4. **FileInfo Usage**

**Code defines:**

```rust
#[derive(Debug)]
struct FileInfo {
    filename: String,
    path: String,
    original_filename: Option<String>,
    field_name: Option<String>,
}
```

But when creating FileInfo:

```rust
let file_info = FileInfo {
    filename: filename.clone(),
    path: filename_with_path.clone(),
    original_filename: original_filename.clone(), // This is always None from the tuple destructuring above
    field_name: field_name.clone(),
};
```

**Issue:** The original_filename is not properly passed through from the multipart parsing.

## Corrected Documentation

### How File Upload Actually Works

The file upload middleware:

1. **Processes binary requests** - Works with any request body content
2. **Supports multipart forms** - Extracts ALL file parts and text fields from `multipart/form-data`
3. **Detects file extensions** - Uses the `infer` crate for automatic type detection
4. **Generates unique filenames** - Creates UUID-based names to prevent conflicts
5. **Saves files** - Writes uploaded content to the specified directory
6. **Maps form fields** - File input field names are mapped to generated UUID filenames in form data
7. **Sets request data** - Adds comprehensive file information for route handlers

### Form Field Behavior

For multipart forms:

- **Text fields**: Available via `req.form_data()` with original names and values
- **File fields**: Field names are mapped to generated UUID filenames in `req.form_data()`
- **Example**: If you have `<input name="profile_pic" type="file">`, then `req.form_data().get("profile_pic")` returns the UUID filename like `"a1b2c3d4-e5f6-7890-abcd-ef1234567890.jpg"`

### Request Data Available

When files are successfully uploaded:

**In req.get_data():**

- `uploaded_file_count` - Number of files successfully uploaded
- `uploaded_files` - JSON array of file info
- `uploaded_file` - First file's UUID filename (backwards compatibility)
- `uploaded_file_path` - First file's full path (backwards compatibility)
- `original_filename` - First file's original name if available from multipart

**In req.form_data():**

- Text field names → their values
- File field names → their generated UUID filenames

### Limitations

1. **Original filenames**: Currently not properly preserved due to tuple handling bug
2. **Single binary uploads**: Always use "file" as the field name
3. **Field mapping**: Only maps the field name to UUID filename, original filename info is lost in form_data

## Recommended Code Fixes

To fix the original filename issue:

```rust
// Change the file_parts tuple to include original filename
type FilePart = (Vec<u8>, Option<String>, Option<String>); // (bytes, field_name, original_filename)

// Update the multipart parser return type
fn parse_multipart_form(
    body: &[u8],
    boundary: &str,
) -> (Vec<(String, String)>, Vec<FilePart>) {
    // ... existing code ...

    if is_file_part {
        let file_bytes = trim_trailing_crlf(&body[content_start..content_end]).to_vec();
        file_parts.push((file_bytes, field_name, original_filename));
    }
    // ... rest of function
}

// Update the processing loop
for (file_bytes, field_name, original_filename) in files_to_process {
    // Now original_filename is properly available
    let file_info = FileInfo {
        filename: filename.clone(),
        path: filename_with_path.clone(),
        original_filename: original_filename.clone(),
        field_name: field_name.clone(),
    };
    // ... rest of processing
}
```
