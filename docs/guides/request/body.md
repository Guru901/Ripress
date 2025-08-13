# RequestBody Documentation

A structured representation of HTTP request body data for Rust web applications.

## Overview

`RequestBody` provides a type-safe wrapper around different types of request body content, automatically managing content types and providing convenient constructors for common body formats used in HTTP requests. It includes two main data types: `TextData` for handling plain text with encoding validation, and `FormData` for managing key-value pairs in form submissions.

## Features

- **Type Safety**: Enforces consistency between content type and actual data format
- **Multiple Formats**: Support for JSON, Form Data, Plain Text, and Empty bodies
- **Automatic Content Types**: Each constructor sets appropriate MIME type headers
- **UTF-8 Validation**: `TextData` provides both strict and lossy UTF-8 handling
- **Form Parsing**: `FormData` supports URL-encoded and comma-separated formats
- **Memory Efficient**: Only stores data needed for each content type

## RequestBody API Reference

### Construction

#### `new_text(text: TextData) -> Self`

Creates a request body with plain text content and `text/plain` content type.

```rust
use ripress::req::body::{RequestBody, TextData};

let text_data = TextData::new(String::from("Hello, server!"));
let body = RequestBody::new_text(text_data);
```

#### `new_form(form_data: FormData) -> Self`

Creates a request body with form data content and `application/x-www-form-urlencoded` content type.

```rust
use ripress::req::body::{RequestBody, FormData};

let mut form = FormData::new();
form.insert("username", "alice");
form.insert("password", "secret123");
let body = RequestBody::new_form(form);
```

#### `new_json<T: Into<serde_json::Value>>(json: T) -> Self`

Creates a request body with JSON content and `application/json` content type.

```rust
use ripress::req::body::RequestBody;
use serde_json::json;

// Using json! macro
let body = RequestBody::new_json(json!({
    "username": "alice",
    "email": "alice@example.com"
}));

// Using serializable structs
#[derive(serde::Serialize)]
struct User { id: u64, name: String }
let user = User { id: 123, name: "Bob".to_string() };
let body = RequestBody::new_json(serde_json::to_value(user).unwrap());
```

### Content Types

The `RequestBodyType` enum provides type-safe content type handling:

- `JSON` → `"application/json"`
- `TEXT` → `"text/plain"`
- `FORM` → `"application/x-www-form-urlencoded"`
- `EMPTY` → `""` (no content)

```rust
use ripress::req::body::RequestBodyType;

assert_eq!(RequestBodyType::JSON.to_string(), "application/json");
assert_eq!(RequestBodyType::FORM.to_string(), "application/x-www-form-urlencoded");
```

---

## TextData API Reference

A flexible container for text data that handles both valid UTF-8 and raw bytes with encoding validation.

### Construction

#### `new(text: String) -> Self`

Creates `TextData` from a `String` (guaranteed valid UTF-8).

```rust
use ripress::req::body::TextData;

let text = TextData::new("Hello, world!".to_string());
assert_eq!(text.charset(), Some("utf-8"));
assert!(text.is_valid_utf8());
```

#### `from_bytes(bytes: Vec<u8>) -> Result<Self, TextDataError>`

Creates `TextData` from bytes with UTF-8 validation.

```rust
use ripress::req::body::TextData;

// Valid UTF-8
let bytes = "Hello, 世界!".as_bytes().to_vec();
let text = TextData::from_bytes(bytes).unwrap();
assert_eq!(text.as_str().unwrap(), "Hello, 世界!");

// Invalid UTF-8
let invalid_bytes = vec![0xFF, 0xFE];
assert!(TextData::from_bytes(invalid_bytes).is_err());
```

#### `from_bytes_with_limit(bytes: Vec<u8>, limit: usize) -> Result<Self, TextDataError>`

Creates `TextData` with size validation and UTF-8 checking.

```rust
use ripress::req::body::TextData;

let small_text = "Hi!".as_bytes().to_vec();
let text = TextData::from_bytes_with_limit(small_text, 10).unwrap();
assert_eq!(text.len_bytes(), 3);

let large_text = "This is a very long string".as_bytes().to_vec();
assert!(TextData::from_bytes_with_limit(large_text, 5).is_err());
```

#### `from_raw_bytes(bytes: Vec<u8>, charset: Option<String>) -> Self`

Creates `TextData` without UTF-8 validation (for non-UTF-8 encodings).

```rust
use ripress::req::body::TextData;

let raw_bytes = vec![0xC4, 0xE9, 0xF1, 0xF2]; // Some encoding
let text = TextData::from_raw_bytes(raw_bytes, Some("cp1252".to_string()));
assert_eq!(text.charset(), Some("cp1252"));
assert!(!text.is_valid_utf8());
```

### Text Access

#### `as_str(&self) -> Result<&str, TextDataError>`

Returns text as string slice with UTF-8 validation.

```rust
let text = TextData::new("Hello!".to_string());
assert_eq!(text.as_str().unwrap(), "Hello!");

let invalid = TextData::from_raw_bytes(vec![0xFF], None);
assert!(invalid.as_str().is_err());
```

#### `as_str_lossy(&self) -> std::borrow::Cow<str>`

Returns text as string, replacing invalid UTF-8 with replacement characters (never fails).

```rust
let valid = TextData::new("Hello!".to_string());
assert_eq!(valid.as_str_lossy(), "Hello!");

let invalid = TextData::from_raw_bytes(vec![b'H', b'i', 0xFF], None);
assert_eq!(invalid.as_str_lossy(), "Hi�");
```

#### `into_string(self) -> Result<String, TextDataError>`

Consumes `TextData` and converts to `String` with UTF-8 validation.

```rust
let text = TextData::new("Hello!".to_string());
let string = text.into_string().unwrap();
assert_eq!(string, "Hello!");
```

#### `into_string_lossy(self) -> String`

Consumes `TextData` and converts to `String` with lossy conversion (never fails).

```rust
let invalid = TextData::from_raw_bytes(vec![b'H', b'i', 0xFF], None);
assert_eq!(invalid.into_string_lossy(), "Hi�");
```

### Byte Access

#### `as_bytes(&self) -> &[u8]`

Returns reference to underlying byte array.

```rust
let text = TextData::new("Hello!".to_string());
assert_eq!(text.as_bytes(), b"Hello!");
```

#### `into_bytes(self) -> Vec<u8>`

Consumes `TextData` and returns underlying byte vector.

```rust
let text = TextData::new("Hello!".to_string());
let bytes = text.into_bytes();
assert_eq!(bytes, b"Hello!");
```

### Size and Validation

#### `len_bytes(&self) -> usize`

Returns length in bytes (may differ from character count for UTF-8).

```rust
let ascii = TextData::new("Hello".to_string());
assert_eq!(ascii.len_bytes(), 5);

let unicode = TextData::new("世界".to_string()); // Two Chinese characters
assert_eq!(unicode.len_bytes(), 6); // 3 bytes each in UTF-8
```

#### `len_chars(&self) -> Result<usize, TextDataError>`

Returns length in Unicode scalar values (characters).

```rust
let ascii = TextData::new("Hello".to_string());
assert_eq!(ascii.len_chars().unwrap(), 5);

let unicode = TextData::new("🦀Rust".to_string());
assert_eq!(unicode.len_chars().unwrap(), 5); // 1 emoji + 4 ASCII chars
assert_eq!(unicode.len_bytes(), 8); // But 8 bytes total
```

#### `is_empty(&self) -> bool`

Returns true if the text data contains no bytes.

```rust
let empty = TextData::new(String::new());
assert!(empty.is_empty());
```

#### `is_valid_utf8(&self) -> bool`

Returns true if underlying bytes form valid UTF-8.

```rust
let valid = TextData::new("Hello, 世界!".to_string());
assert!(valid.is_valid_utf8());

let invalid = TextData::from_raw_bytes(vec![0xFF, 0xFE], None);
assert!(!invalid.is_valid_utf8());
```

### Charset Management

#### `charset(&self) -> Option<&str>`

Returns charset information if available.

```rust
let utf8_text = TextData::new("Hello".to_string());
assert_eq!(utf8_text.charset(), Some("utf-8"));

let raw_text = TextData::from_raw_bytes(vec![0x48, 0x69], None);
assert_eq!(raw_text.charset(), None);
```

#### `set_charset(&mut self, charset: String)`

Sets charset information (metadata only, doesn't validate encoding).

```rust
let mut text = TextData::from_raw_bytes(vec![0xE9], None);
text.set_charset("iso-8859-1".to_string());
assert_eq!(text.charset(), Some("iso-8859-1"));
```

### Text Processing

#### `lines(&self) -> Result<std::str::Lines, TextDataError>`

Returns iterator over lines (split on `\n`).

```rust
let text = TextData::new("Line 1\nLine 2\nLine 3".to_string());
let lines: Vec<&str> = text.lines().unwrap().collect();
assert_eq!(lines, vec!["Line 1", "Line 2", "Line 3"]);
```

#### `trim(&self) -> Result<&str, TextDataError>`

Returns text with leading/trailing whitespace removed.

```rust
let text = TextData::new("  Hello, world!  ".to_string());
assert_eq!(text.trim().unwrap(), "Hello, world!");
```

#### `contains(&self, needle: &str) -> Result<bool, TextDataError>`

Returns true if text contains the specified substring.

```rust
let text = TextData::new("Hello, world!".to_string());
assert!(text.contains("world").unwrap());
assert!(!text.contains("Rust").unwrap());
```

#### `split<'a>(&'a self, delimiter: &'a str) -> Result<std::str::Split<'a, &'a str>, TextDataError>`

Returns iterator over substrings split by delimiter.

```rust
let text = TextData::new("apple,banana,cherry".to_string());
let parts: Vec<&str> = text.split(",").unwrap().collect();
assert_eq!(parts, vec!["apple", "banana", "cherry"]);
```

### Truncation

#### `truncate_bytes(&mut self, max_len: usize)`

Truncates data to maximum byte length (respects UTF-8 boundaries).

```rust
let mut text = TextData::new("Hello, world!".to_string());
text.truncate_bytes(5);
assert_eq!(text.as_str().unwrap(), "Hello");

// Safe with multi-byte UTF-8
let mut unicode_text = TextData::new("世界Hello".to_string());
unicode_text.truncate_bytes(7);
assert!(unicode_text.is_valid_utf8()); // Still valid after truncation
```

#### `truncated_bytes(&self, max_len: usize) -> Self`

Returns truncated copy (original unchanged).

```rust
let text = TextData::new("Hello, world!".to_string());
let short = text.truncated_bytes(5);
assert_eq!(short.as_str().unwrap(), "Hello");
assert_eq!(text.as_str().unwrap(), "Hello, world!"); // Original unchanged
```

### Type Conversions

```rust
// From String/&str
let text1 = TextData::from("Hello");
let text2 = TextData::from("World".to_string());

// Try from Vec<u8>
let text3 = TextData::try_from("Hello".as_bytes().to_vec()).unwrap();

// To String
let string: String = text1.try_into().unwrap();

// Deref to &[u8]
let bytes: &[u8] = &*text2;
```

---

## FormData API Reference

A convenient wrapper around `HashMap<String, String>` for handling form data with parsing capabilities.

### Construction

#### `new() -> Self`

Creates new empty `FormData`.

```rust
use ripress::req::body::FormData;

let form = FormData::new();
assert!(form.is_empty());
```

#### `with_capacity(capacity: usize) -> Self`

Creates `FormData` with specified capacity.

```rust
let form = FormData::with_capacity(10);
assert!(form.is_empty());
```

#### `from_map(map: HashMap<String, String>) -> Self`

Creates from existing HashMap.

```rust
use std::collections::HashMap;

let mut map = HashMap::new();
map.insert("key".to_string(), "value".to_string());
let form = FormData::from_map(map);
assert_eq!(form.get("key"), Some("value"));
```

### Data Manipulation

#### `insert<K, V>(&mut self, key: K, value: V) -> Option<String>`

Inserts key-value pair, replacing existing value if present.

```rust
let mut form = FormData::new();
assert_eq!(form.insert("key", "value"), None);
assert_eq!(form.insert("key", "new_value"), Some("value".to_string()));
```

#### `append<K, V>(&mut self, key: K, value: V)`

Appends value to existing key (creates comma-separated list).

```rust
let mut form = FormData::new();
form.append("tags", "rust");
form.append("tags", "web");
assert_eq!(form.get("tags"), Some("rust,web"));
```

#### `extend<I, K, V>(&mut self, iter: I)`

Extends with key-value pairs from iterator.

```rust
let mut form = FormData::new();
let pairs = vec![("a", "1"), ("b", "2")];
form.extend(pairs);
assert_eq!(form.len(), 2);
```

### Data Access

#### `get(&self, key: &str) -> Option<&str>`

Gets value for key.

```rust
let mut form = FormData::new();
form.insert("key", "value");
assert_eq!(form.get("key"), Some("value"));
assert_eq!(form.get("missing"), None);
```

#### `get_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str`

Gets value or default if key doesn't exist.

```rust
let mut form = FormData::new();
form.insert("key", "value");
assert_eq!(form.get_or("key", "default"), "value");
assert_eq!(form.get_or("missing", "default"), "default");
```

#### `get_mut(&mut self, key: &str) -> Option<&mut String>`

Gets mutable reference to value.

```rust
let mut form = FormData::new();
form.insert("key", "value");
if let Some(value) = form.get_mut("key") {
    value.push_str("_modified");
}
assert_eq!(form.get("key"), Some("value_modified"));
```

### Collection Operations

#### `remove(&mut self, key: &str) -> Option<String>`

Removes and returns value for key.

```rust
let mut form = FormData::new();
form.insert("key", "value");
assert_eq!(form.remove("key"), Some("value".to_string()));
assert_eq!(form.remove("key"), None);
```

#### `contains_key(&self, key: &str) -> bool`

Checks if key exists.

```rust
let mut form = FormData::new();
form.insert("key", "value");
assert!(form.contains_key("key"));
assert!(!form.contains_key("missing"));
```

#### `clear(&mut self)`

Removes all key-value pairs.

```rust
let mut form = FormData::new();
form.insert("key", "value");
form.clear();
assert!(form.is_empty());
```

#### `retain<F>(&mut self, f: F)`

Retains only pairs satisfying predicate.

```rust
let mut form = FormData::new();
form.insert("keep", "yes");
form.insert("remove", "no");

form.retain(|key, _| key.starts_with("keep"));
assert!(form.contains_key("keep"));
assert!(!form.contains_key("remove"));
```

### Inspection

#### `len(&self) -> usize`

Returns number of key-value pairs.

```rust
let mut form = FormData::new();
assert_eq!(form.len(), 0);
form.insert("key", "value");
assert_eq!(form.len(), 1);
```

#### `is_empty(&self) -> bool`

Returns true if no pairs exist.

```rust
let mut form = FormData::new();
assert!(form.is_empty());
form.insert("key", "value");
assert!(!form.is_empty());
```

#### `as_map(&self) -> &HashMap<String, String>`

Returns reference to underlying HashMap.

```rust
let form = FormData::new();
let map = form.as_map();
assert!(map.is_empty());
```

### Iteration

#### `keys(&self) -> impl Iterator<Item = &str>`

Iterates over keys.

```rust
let mut form = FormData::new();
form.insert("a", "1");
form.insert("b", "2");

let keys: Vec<_> = form.keys().collect();
assert_eq!(keys.len(), 2);
```

#### `values(&self) -> impl Iterator<Item = &str>`

Iterates over values.

```rust
let mut form = FormData::new();
form.insert("a", "1");
form.insert("b", "2");

let values: Vec<_> = form.values().collect();
assert_eq!(values.len(), 2);
```

#### `iter(&self) -> impl Iterator<Item = (&str, &str)>`

Iterates over key-value pairs.

```rust
let mut form = FormData::new();
form.insert("a", "1");
form.insert("b", "2");

for (key, value) in form.iter() {
    println!("{}: {}", key, value);
}
```

### Parsing and Serialization

#### `to_query_string(&self) -> String`

Converts to URL-encoded query string.

```rust
let mut form = FormData::new();
form.insert("name", "John Doe");
form.insert("age", "30");

let query = form.to_query_string();
// Order may vary due to HashMap
assert!(query.contains("name=John%20Doe"));
assert!(query.contains("age=30"));
```

#### `from_query_string(query: &str) -> Result<Self, String>`

Parses URL-encoded query string.

```rust
let form = FormData::from_query_string("name=John%20Doe&age=30").unwrap();
assert_eq!(form.get("name"), Some("John Doe"));
assert_eq!(form.get("age"), Some("30"));

// Handles keys without values
let form2 = FormData::from_query_string("flag&other=value").unwrap();
assert_eq!(form2.get("flag"), Some(""));
assert_eq!(form2.get("other"), Some("value"));
```

#### `from_comma_separated(query: &str) -> Result<Self, String>`

Parses comma-separated key-value pairs.

```rust
// Comma-space separated
let form = FormData::from_comma_separated("name=Alice, age=30, city=Boston").unwrap();
assert_eq!(form.get("name"), Some("Alice"));
assert_eq!(form.get("age"), Some("30"));
assert_eq!(form.get("city"), Some("Boston"));

// Falls back to ampersand if no comma-space found
let form2 = FormData::from_comma_separated("name=Bob&age=25").unwrap();
assert_eq!(form2.get("name"), Some("Bob"));

// Handles URL encoding
let form3 = FormData::from_comma_separated("name=John%20Doe, location=New%20York").unwrap();
assert_eq!(form3.get("name"), Some("John Doe"));
assert_eq!(form3.get("location"), Some("New York"));
```

### Special Syntax

#### Index Access

```rust
let mut form_data = FormData::new();
form_data.insert("username", "alice");
assert_eq!(&form_data["username"], "alice");
// Note: Panics if key doesn't exist! Use get() for safe access.
```

#### Display Formatting

```rust
let mut form = FormData::new();
form.insert("name", "Alice");
form.insert("age", "30");

println!("{}", form);
// Output: name=Alice, age=30 (order may vary)
```

#### Conversion Traits

```rust
// From HashMap
let map: HashMap<String, String> = [
    ("key1".to_string(), "value1".to_string()),
    ("key2".to_string(), "value2".to_string()),
].into();
let form = FormData::from(map);

// To HashMap
let form = FormData::new();
let map: HashMap<String, String> = form.into();

// From iterator
let pairs = vec![("name", "Alice"), ("age", "30")];
let form: FormData = pairs.into_iter().collect();

// Iterator support
for (key, value) in &form {
    println!("{}: {}", key, value);
}

for (key, value) in form {
    println!("Owned: {}: {}", key, value);
}
```

## Error Handling

### TextDataError

```rust
use ripress::req::body::TextDataError;

match text_result {
    Err(TextDataError::InvalidUtf8(e)) => {
        eprintln!("Invalid UTF-8: {}", e);
    }
    Err(TextDataError::TooLarge { size, limit }) => {
        eprintln!("Text too large: {} bytes (limit: {})", size, limit);
    }
    Err(TextDataError::Empty) => {
        eprintln!("Text data is empty");
    }
    Ok(text) => {
        println!("Valid text: {}", text.as_str().unwrap());
    }
}
```

## Example: Complete Request Processing

```rust
use ripress::req::body::{RequestBody, RequestBodyContent, FormData, TextData};
use serde_json::json;

fn process_request(body: RequestBody) -> String {
    match body.content {
        RequestBodyContent::JSON(json_value) => {
            format!("Processing JSON: {}", json_value)
        }
        RequestBodyContent::FORM(form_data) => {
            let mut result = String::from("Form data received:\n");
            for (key, value) in form_data.iter() {
                result.push_str(&format!("  {}: {}\n", key, value));
            }
            result
        }
        RequestBodyContent::TEXT(text_data) => {
            match text_data.as_str() {
                Ok(text) => format!("Text content: {}", text),
                Err(_) => format!("Text content (lossy): {}", text_data.as_str_lossy()),
            }
        }
        RequestBodyContent::EMPTY => {
            String::from("Empty request body")
        }
    }
}

// Usage examples
let json_body = RequestBody::new_json(json!({"action": "login", "user": "alice"}));
let result1 = process_request(json_body);

let mut form = FormData::new();
form.insert("username", "bob");
form.insert("email", "bob@example.com");
let form_body = RequestBody::new_form(form);
let result2 = process_request(form_body);

let text = TextData::new("Raw log data".to_string());
let text_body = RequestBody::new_text(text);
let result3 = process_request(text_body);
```

## Thread Safety

All types (`RequestBody`, `TextData`, `FormData`) implement `Clone` and are `Send + Sync`, making them safe to share across threads. For concurrent mutation, use standard Rust synchronization primitives like `Mutex` or `RwLock`.

## Performance Considerations

- **TextData**: UTF-8 validation occurs on creation and access methods. Use `as_str_lossy()` to avoid validation overhead when acceptable
- **FormData**: Uses `HashMap` internally for O(1) average-case lookups. Memory usage scales with number of key-value pairs
- **RequestBody**: Memory efficient enum design - only stores data relevant to the specific content type
- **Parsing**: URL decoding and parsing operations allocate new strings. Consider caching parsed results for frequently accessed data
