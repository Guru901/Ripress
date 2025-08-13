# RequestData Documentation

A high-performance byte-oriented data structure for Rust applications handling mixed text and binary data.

## Overview

`RequestData` provides an efficient key-value storage system optimized for scenarios where you need to handle arbitrary byte data, such as HTTP request processing, form data handling, or general-purpose byte storage. It uses `ByteKey` internally for optimal performance with byte-based keys.

## Features

- **Flexible data types**: Handles both text and binary data seamlessly
- **Memory efficient**: Optimized for minimal allocations and memory usage
- **UTF-8 aware**: Automatic string conversion with fallback to raw bytes
- **High performance**: Uses `HashMap` with optimized byte-based keys
- **Type flexibility**: Generic methods accept any type implementing `AsRef<[u8]>`
- **Memory management**: Built-in capacity management and size tracking
- **Iterator support**: Multiple ways to iterate over stored data
- **Display formatting**: Pretty-print data for debugging

## Basic Usage

```rust
use ripress::req::request_data::{RequestData, ByteKey};

// Create new empty data container
let mut data = RequestData::new();

// Insert various types of data
data.insert("username", "john_doe");
data.insert("session_id", "abc123xyz789");
data.insert("file_data", &[0xFF, 0xD8, 0xFF, 0xE0]); // Binary data

// Insert owned data (more efficient if you have Vec<u8>)
let key = b"binary_key".to_vec();
let value = vec![1, 2, 3, 4, 5];
data.insert_owned(key, value);

// Retrieve data as UTF-8 strings
if let Some(username) = data.get("username") {
    println!("Username: {}", username);
}

// Check if data exists
if data.contains_key("session_id") {
    println!("User has active session");
}

// Monitor memory usage
println!("Data size: {} bytes", data.byte_size());
```

## API Reference

### Construction

#### `new() -> Self`

Creates a new empty `RequestData` collection.

```rust
let data = RequestData::new();
```

#### `with_capacity(capacity: usize) -> Self`

Creates a new empty `RequestData` collection with pre-allocated capacity.

**Parameters:**

- `capacity`: Initial capacity for the internal HashMap

```rust
let data = RequestData::with_capacity(100);
```

#### `default() -> Self`

Creates a new empty `RequestData` collection (implements `Default` trait).

```rust
let data = RequestData::default();
```

### Data Manipulation

#### `insert<K, V>(&mut self, key: K, value: V)`

Inserts data, replacing any existing value for the key.

**Parameters:**

- `key`: Key data (anything implementing `AsRef<[u8]>`)
- `value`: Value data (anything implementing `AsRef<[u8]>`)

```rust
data.insert("name", "John Doe");
data.insert("id", "12345");
data.insert("binary", &[0x48, 0x65, 0x6C, 0x6C, 0x6F]); // "Hello" in bytes
```

#### `insert_owned(&mut self, key: Vec<u8>, value: Vec<u8>)`

Inserts owned byte vectors without copying (more efficient for `Vec<u8>` data).

**Parameters:**

- `key`: Owned key bytes
- `value`: Owned value bytes

```rust
let key = b"image_data".to_vec();
let image_bytes = load_image_bytes();
data.insert_owned(key, image_bytes);
```

#### `remove<K>(&mut self, key: K) -> Option<Vec<u8>>`

Removes and returns the raw bytes for a key.

**Parameters:**

- `key`: Key to remove

**Returns:** `Option<Vec<u8>>` - The removed value bytes, if they existed

```rust
if let Some(removed_data) = data.remove("temp_file") {
    println!("Removed {} bytes", removed_data.len());
}
```

### Data Access

#### `get<K>(&self, key: K) -> Option<String>`

Gets a value as a UTF-8 string (most common use case).

**Parameters:**

- `key`: Key to look up

**Returns:** `Option<String>` - The value as a string if it exists and is valid UTF-8

```rust
if let Some(username) = data.get("username") {
    println!("Hello, {}", username);
} else {
    println!("Username not found or not valid UTF-8");
}
```

#### `contains_key<K>(&self, key: K) -> bool`

Checks if a key exists in the data.

**Parameters:**

- `key`: Key to check

**Returns:** `bool` - True if the key exists

```rust
if data.contains_key("auth_token") {
    process_authenticated_request();
} else {
    return_unauthorized_error();
}
```

### Inspection and Iteration

#### `len(&self) -> usize`

Returns the number of key-value pairs.

```rust
println!("Data contains {} items", data.len());
```

#### `is_empty(&self) -> bool`

Checks if there are no key-value pairs.

```rust
if data.is_empty() {
    println!("No data received");
}
```

#### `clear(&mut self)`

Removes all data while preserving allocated capacity.

```rust
data.clear();
assert!(data.is_empty());
```

#### `iter(&self) -> impl Iterator<Item = (&[u8], &[u8])>`

Iterates over key-value pairs as byte slices.

```rust
for (key_bytes, value_bytes) in data.iter() {
    // Handle raw byte data
    if let Ok(key_str) = std::str::from_utf8(key_bytes) {
        println!("Key: {}", key_str);
    }
}
```

#### `keys(&self) -> impl Iterator<Item = &[u8]>`

Iterates over keys as byte slices.

```rust
for key_bytes in data.keys() {
    if let Ok(key_str) = std::str::from_utf8(key_bytes) {
        println!("Key: {}", key_str);
    }
}
```

#### `values(&self) -> impl Iterator<Item = &[u8]>`

Iterates over values as byte slices.

```rust
let total_size: usize = data.values()
    .map(|value| value.len())
    .sum();
println!("Total data size: {} bytes", total_size);
```

### Utility Methods

#### `from_map<K, V>(map: HashMap<K, V>) -> Self`

Creates `RequestData` from an existing HashMap.

**Parameters:**

- `map`: HashMap where keys and values implement `AsRef<[u8]>`

**Returns:** New `RequestData` instance

```rust
use std::collections::HashMap;

let mut map = HashMap::new();
map.insert("key1", "value1");
map.insert("key2", "value2");

let data = RequestData::from_map(map);
```

#### `byte_size(&self) -> usize`

Returns approximate total memory usage in bytes.

**Returns:** `usize` - Estimated memory usage including HashMap overhead

```rust
let size = data.byte_size();
println!("Memory usage: {} bytes", size);

if size > 1024 * 1024 {  // 1MB
    println!("Large dataset detected");
}
```

#### `shrink_to_fit(&mut self)`

Reduces memory usage by shrinking capacity to fit current data.

```rust
// After removing many items
data.shrink_to_fit();
println!("Optimized memory usage: {} bytes", data.byte_size());
```

### Special Syntax

#### Consuming Iterator

You can iterate over owned data by consuming the `RequestData`:

```rust
for (key_bytes, value_bytes) in data {
    // Now you own the Vec<u8> data
    let key = String::from_utf8_lossy(&key_bytes);
    let value = String::from_utf8_lossy(&value_bytes);
    println!("{}: {}", key, value);
}
// `data` is no longer accessible here
```

#### Display Formatting

The struct implements `Display` for easy debugging:

```rust
println!("Current data: {}", data);
// Output: RequestData { username: john_doe, session_id: abc123xyz789, file_data: [255, 216, 255, 224] }
```

## ByteKey Helper Type

The `ByteKey` type is used internally but can also be useful in your own code:

```rust
use ripress::req::request_data::ByteKey;

// Create keys from various data types
let key1 = ByteKey::new("string_key");
let key2 = ByteKey::new(b"byte_key");
let key3 = ByteKey::new(vec![1, 2, 3]);

// Convert back to string if valid UTF-8
if let Ok(s) = key1.as_str() {
    println!("Key as string: {}", s);
}

// Always access as bytes
let bytes = key1.as_bytes();
println!("Key bytes: {:?}", bytes);
```

## Example: HTTP Form Data Processing

```rust
fn process_form_data(raw_data: &[u8]) -> RequestData {
    let mut data = RequestData::new();

    // Parse hypothetical form data format
    for line in raw_data.split(|&b| b == b'\n') {
        if let Some(eq_pos) = line.iter().position(|&b| b == b'=') {
            let key = &line[..eq_pos];
            let value = &line[eq_pos + 1..];
            data.insert(key, value);
        }
    }

    data
}

fn handle_request(form_data: RequestData) {
    // Extract text fields
    let username = form_data.get("username").unwrap_or_default();
    let email = form_data.get("email").unwrap_or_default();

    // Check for binary data
    if form_data.contains_key("profile_image") {
        println!("Profile image uploaded");
    }

    // Log request size
    println!("Form data size: {} bytes", form_data.byte_size());

    // Process each field
    for (key_bytes, value_bytes) in form_data.iter() {
        let field_name = String::from_utf8_lossy(key_bytes);

        if value_bytes.len() > 1024 {
            println!("Large field '{}': {} bytes", field_name, value_bytes.len());
        } else if let Ok(text_value) = std::str::from_utf8(value_bytes) {
            println!("Text field '{}': {}", field_name, text_value);
        } else {
            println!("Binary field '{}': {} bytes", field_name, value_bytes.len());
        }
    }
}
```

## Example: Session Data Storage

```rust
use std::collections::HashMap;

struct SessionManager {
    sessions: HashMap<String, RequestData>,
}

impl SessionManager {
    fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    fn create_session(&mut self, session_id: String) -> &mut RequestData {
        self.sessions.entry(session_id).or_insert_with(RequestData::new)
    }

    fn store_user_data(&mut self, session_id: &str, key: &str, value: &str) {
        if let Some(session_data) = self.sessions.get_mut(session_id) {
            session_data.insert(key, value);
        }
    }

    fn get_user_data(&self, session_id: &str, key: &str) -> Option<String> {
        self.sessions.get(session_id)?.get(key)
    }

    fn session_size(&self, session_id: &str) -> usize {
        self.sessions.get(session_id)
            .map(|data| data.byte_size())
            .unwrap_or(0)
    }
}

// Usage
let mut manager = SessionManager::new();
let session_data = manager.create_session("user123".to_string());

session_data.insert("username", "alice");
session_data.insert("preferences", r#"{"theme":"dark","lang":"en"}"#);
session_data.insert("avatar", &[0xFF, 0xD8, 0xFF, 0xE0]); // Binary image data

println!("Session size: {} bytes", manager.session_size("user123"));
```

## Thread Safety

`RequestData` and `ByteKey` both derive `Clone` and `Debug`, making them easy to share across threads. The internal `HashMap<ByteKey, Vec<u8>>` is `Send + Sync`, so the structs can be shared across threads following standard Rust ownership rules.

For concurrent access, wrap in your preferred synchronization primitive:

```rust
use std::sync::{Arc, Mutex};

let shared_data = Arc::new(Mutex::new(RequestData::new()));

// In different threads...
let data = shared_data.clone();
std::thread::spawn(move || {
    let mut data = data.lock().unwrap();
    data.insert("thread_data", "from worker thread");
});
```

## Performance Considerations

- **Memory efficiency**: Stores data as `Vec<u8>` for minimal overhead
- **Key performance**: `ByteKey` uses efficient byte-based hashing and equality
- **Lookup speed**: O(1) average-case lookup performance via `HashMap`
- **Insertion cost**: Keys and values are copied on insertion unless using `insert_owned`
- **Memory usage**: Scales linearly with the amount of stored data
- **Capacity management**: Use `with_capacity` for known data sizes and `shrink_to_fit` to optimize memory

### Performance Tips

1. **Use `insert_owned`** when you have `Vec<u8>` data to avoid extra allocations
2. **Pre-allocate capacity** with `with_capacity` if you know the approximate number of items
3. **Call `shrink_to_fit`** after removing many items to reclaim memory
4. **Monitor size** with `byte_size()` for memory-sensitive applications

```rust
// Efficient bulk insertion
let mut data = RequestData::with_capacity(1000);
for (key, value) in large_dataset {
    if let (Ok(key_bytes), Ok(value_bytes)) = (key.into_bytes(), value.into_bytes()) {
        data.insert_owned(key_bytes, value_bytes); // No extra copying
    }
}
```

## Error Handling

`RequestData` uses `Option` types for safe access patterns:

```rust
fn safe_data_access(data: &RequestData) {
    // Safe string access
    match data.get("user_input") {
        Some(value) => println!("User input: {}", value),
        None => println!("No user input provided"),
    }

    // Safe existence check
    if data.contains_key("required_field") {
        process_required_data();
    } else {
        return_validation_error();
    }

    // Safe removal
    if let Some(temp_data) = data.remove("temporary") {
        cleanup_temporary_data(temp_data);
    }
}
```

## Integration Examples

### Web Framework Integration

```rust
// Example integration with a web framework
use serde_json::Value;

fn parse_request_body(content_type: &str, body: &[u8]) -> RequestData {
    let mut data = RequestData::new();

    match content_type {
        "application/json" => {
            if let Ok(json) = serde_json::from_slice::<Value>(body) {
                flatten_json(&json, &mut data, "");
            }
        },
        "application/x-www-form-urlencoded" => {
            parse_form_encoded(body, &mut data);
        },
        "multipart/form-data" => {
            parse_multipart(body, &mut data);
        },
        _ => {
            data.insert("raw_body", body);
        }
    }

    data
}

fn flatten_json(value: &Value, data: &mut RequestData, prefix: &str) {
    match value {
        Value::Object(obj) => {
            for (key, val) in obj {
                let full_key = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                flatten_json(val, data, &full_key);
            }
        },
        Value::String(s) => {
            data.insert(prefix, s);
        },
        _ => {
            data.insert(prefix, value.to_string());
        }
    }
}
```

### Database Serialization

```rust
// Example: Store RequestData in a database
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct SerializableData {
    items: Vec<(Vec<u8>, Vec<u8>)>,
}

impl From<RequestData> for SerializableData {
    fn from(data: RequestData) -> Self {
        let items = data.into_iter().collect();
        Self { items }
    }
}

impl From<SerializableData> for RequestData {
    fn from(serializable: SerializableData) -> Self {
        let mut data = RequestData::with_capacity(serializable.items.len());
        for (key, value) in serializable.items {
            data.insert_owned(key, value);
        }
        data
    }
}

// Usage
fn save_to_database(data: RequestData) -> Result<(), DatabaseError> {
    let serializable: SerializableData = data.into();
    let json = serde_json::to_string(&serializable)?;
    database.store("request_data", &json)
}

fn load_from_database() -> Result<RequestData, DatabaseError> {
    let json = database.retrieve("request_data")?;
    let serializable: SerializableData = serde_json::from_str(&json)?;
    Ok(serializable.into())
}
```

### Caching Layer

```rust
use std::time::{Duration, Instant};

struct CachedRequestData {
    data: RequestData,
    created_at: Instant,
    ttl: Duration,
}

impl CachedRequestData {
    fn new(data: RequestData, ttl: Duration) -> Self {
        Self {
            data,
            created_at: Instant::now(),
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }

    fn size(&self) -> usize {
        self.data.byte_size()
    }
}

struct RequestDataCache {
    cache: HashMap<String, CachedRequestData>,
    max_size: usize,
}

impl RequestDataCache {
    fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
        }
    }

    fn insert(&mut self, key: String, data: RequestData, ttl: Duration) {
        // Clean expired entries
        self.cleanup_expired();

        // Check size limits
        let data_size = data.byte_size();
        if data_size > self.max_size {
            return; // Too large to cache
        }

        // Make room if needed
        while self.total_size() + data_size > self.max_size {
            if let Some(lru_key) = self.find_lru_key() {
                self.cache.remove(&lru_key);
            } else {
                break;
            }
        }

        self.cache.insert(key, CachedRequestData::new(data, ttl));
    }

    fn get(&mut self, key: &str) -> Option<&RequestData> {
        if let Some(cached) = self.cache.get(key) {
            if cached.is_expired() {
                self.cache.remove(key);
                None
            } else {
                Some(&cached.data)
            }
        } else {
            None
        }
    }

    fn total_size(&self) -> usize {
        self.cache.values().map(|cached| cached.size()).sum()
    }

    fn cleanup_expired(&mut self) {
        let expired_keys: Vec<_> = self.cache
            .iter()
            .filter(|(_, cached)| cached.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            self.cache.remove(&key);
        }
    }

    fn find_lru_key(&self) -> Option<String> {
        self.cache
            .iter()
            .min_by_key(|(_, cached)| cached.created_at)
            .map(|(key, _)| key.clone())
    }
}
```

## Best Practices

1. **Memory Management**: Always consider memory usage in long-running applications

   ```rust
   // Periodically optimize memory usage
   if data.len() < data.byte_size() / 100 {  // Less than 1% utilization
       data.shrink_to_fit();
   }
   ```

2. **Error Handling**: Always handle the case where UTF-8 conversion fails

   ```rust
   match data.get("user_input") {
       Some(text) => process_text(&text),
       None => {
           // Either key doesn't exist or value isn't valid UTF-8
           // Check raw bytes if needed
           if data.contains_key("user_input") {
               handle_binary_data();
           } else {
               handle_missing_data();
           }
       }
   }
   ```

3. **Bulk Operations**: Use appropriate methods for bulk data handling

   ```rust
   // Efficient bulk insertion
   let mut data = RequestData::with_capacity(items.len());
   for (key, value) in items {
       data.insert_owned(key, value);
   }

   // Efficient bulk processing
   let total_size: usize = data.values().map(|v| v.len()).sum();
   ```

4. **Type Safety**: Use strongly-typed wrappers for domain-specific data

   ```rust
   struct UserId(String);
   struct SessionToken(String);

   impl UserId {
       fn from_request_data(data: &RequestData) -> Option<Self> {
           data.get("user_id").map(UserId)
       }
   }
   ```

## Comparison with Alternatives

| Feature             | `RequestData`      | `HashMap<String, String>` | `HashMap<String, Vec<u8>>` |
| ------------------- | ------------------ | ------------------------- | -------------------------- |
| Binary data support | ✅                 | ❌                        | ✅                         |
| UTF-8 conversion    | ✅ Automatic       | ✅ Built-in               | ❌ Manual                  |
| Memory efficiency   | ✅ Optimized       | ❌ String overhead        | ✅ Raw bytes               |
| Flexible keys       | ✅ Any bytes       | ❌ String only            | ❌ String only             |
| Type safety         | ✅ Generic methods | ❌ String-specific        | ❌ Bytes-specific          |
| Display formatting  | ✅ Smart fallback  | ✅ Simple                 | ❌ Raw bytes only          |

`RequestData` provides the best of both worlds: the flexibility to handle binary data with the convenience of automatic string conversion when possible.
