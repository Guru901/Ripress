#![warn(missing_docs)]
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use serde::Serialize;

use crate::error::RipressError;

/// A collection of parameters extracted from a route's URL pattern.
///
/// `RouteParams` provides a type-safe way to handle URL path parameters in web routes.
/// It stores values captured from dynamic route segments and provides convenient methods
/// for retrieving and parsing them into different types.
///
/// # Route Pattern Examples
///
/// Common route patterns that generate parameters:
/// - `/users/:id` - captures `id` parameter
/// - `/posts/:slug/comments/:comment_id` - captures `slug` and `comment_id`
/// - `/api/v1/:resource/:id/details` - captures `resource` and `id`
/// - `/files/:path*` - captures wildcard `path` parameter
///
/// # Key Features
///
/// - **Type-safe parsing**: Convert string parameters to integers, floats, or custom types
/// - **Error handling**: Comprehensive error types for missing and invalid parameters
/// - **Default values**: Fallback mechanisms for optional parameters
/// - **Convenient access**: Index syntax and specialized getters for common use cases
/// - **Validation support**: Built-in methods for parameter validation
/// - **Iterator support**: Traverse all parameters efficiently
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use ripress::req::route_params::RouteParams;
///
/// // Typically populated by the router from a URL like "/users/42"
/// let mut params = RouteParams::new();
/// params.insert("id", "42");
/// params.insert("format", "json");
///
/// // Access as string
/// assert_eq!(params.get("id"), Some("42"));
///
/// // Parse as integer
/// assert_eq!(params.get_int("id").unwrap(), 42);
///
/// // Index syntax for guaranteed access
/// assert_eq!(&params["format"], "json");
/// ```
///
/// ## Type-Safe Parsing
///
/// ```rust
/// use ripress::req::route_params::RouteParams;
///
/// let mut params = RouteParams::new();
/// params.insert("user_id", "123");
/// params.insert("page", "5");
/// params.insert("limit", "20");
///
/// // Parse different types
/// let user_id: i32 = params.get_parsed("user_id").unwrap();
/// let page: u32 = params.get_uint("page").unwrap();
/// let limit: usize = params.get_parsed("limit").unwrap();
///
/// assert_eq!(user_id, 123);
/// assert_eq!(page, 5);
/// assert_eq!(limit, 20);
/// ```
///
/// ## Error Handling
///
/// ```rust
/// use ripress::req::route_params::RouteParams;
///
/// let params = RouteParams::new();
///
/// // Handle missing parameters
/// match params.get_int("missing") {
///     Ok(value) => println!("Value: {}", value),
///     Err(e) => println!("Error: {}", e),
/// }
/// ```
///
/// ## Default Values
///
/// ```rust
/// use ripress::req::route_params::RouteParams;
///
/// let mut params = RouteParams::new();
/// params.insert("page", "invalid");
///
/// // Use defaults for missing or invalid parameters
/// let page = params.get_or_default("page", 1_u32);
/// let limit = params.get_or_default("limit", 10_u32); // missing parameter
///
/// assert_eq!(page, 1);  // parsing failed, used default
/// assert_eq!(limit, 10); // parameter missing, used default
/// ```
///
/// ## Bulk Parameter Extraction
///
/// ```rust
/// use ripress::req::route_params::RouteParams;
///
/// let mut params = RouteParams::new();
/// params.insert("user_id", "42");
/// params.insert("post_id", "123");
///
/// // Extract multiple parameters with validation
/// let result = params.extract(|p| {
///     let mut errors = Vec::new();
///     
///     let user_id = p.get_int("user_id").map_err(|e| errors.push(e));
///     let post_id = p.get_int("post_id").map_err(|e| errors.push(e));
///     
///     if errors.is_empty() { Ok(()) } else { Err(errors) }
/// });
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct RouteParams {
    /// Internal storage mapping parameter names to their string values.
    /// All values are stored as strings and parsed on demand.
    pub(crate) params: HashMap<String, String>,
}

/// Errors that can occur when retrieving or parsing route parameters.
///
/// This enum provides detailed information about parameter access failures,
/// making it easier to provide meaningful error messages to users or logs.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) enum ParamError {
    /// The parameter with the given name does not exist in the route.
    ///
    /// This typically occurs when:
    /// - A required parameter is missing from the URL
    /// - There's a typo in the parameter name
    /// - The route pattern doesn't include the expected parameter
    NotFound(String),

    /// The parameter exists but could not be parsed into the requested type.
    ///
    /// This occurs when:
    /// - Trying to parse non-numeric strings as numbers
    /// - Type conversion fails (e.g., negative number to unsigned type)
    /// - Custom type parsing fails
    ParseError {
        /// The name of the parameter that failed to parse.
        param: String,

        /// The original string value that couldn't be parsed.
        value: String,

        /// The name of the target type that parsing was attempted for.
        target_type: String,
    },
}

impl fmt::Display for ParamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParamError::NotFound(param) => write!(f, "Route parameter '{}' not found", param),
            ParamError::ParseError {
                param,
                value,
                target_type,
            } => {
                write!(
                    f,
                    "Failed to parse parameter '{}' with value '{}' as {}",
                    param, value, target_type
                )
            }
        }
    }
}

impl std::error::Error for ParamError {}

impl RouteParams {
    /// Creates a new, empty `RouteParams` collection.
    ///
    /// This is typically used internally by routing frameworks, but can be
    /// useful for testing or when manually constructing parameter collections.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::route_params::RouteParams;
    ///
    /// let params = RouteParams::new();
    /// assert!(params.is_empty());
    /// assert_eq!(params.len(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
        }
    }

    /// Creates a `RouteParams` instance from an existing `HashMap`.
    ///
    /// This method is useful for converting existing string-to-string mappings
    /// into a `RouteParams` instance, particularly when integrating with other
    /// libraries or when migrating existing code.
    ///
    /// # Parameters
    ///
    /// - `map`: A HashMap containing parameter name-value pairs as strings
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::route_params::RouteParams;
    /// use std::collections::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("id".to_string(), "123".to_string());
    /// map.insert("slug".to_string(), "hello-world".to_string());
    ///
    /// let params = RouteParams::from_map(map);
    /// assert_eq!(params.get("id"), Some("123"));
    /// assert_eq!(params.get("slug"), Some("hello-world"));
    /// ```
    pub fn from_map(map: HashMap<String, String>) -> Self {
        Self { params: map }
    }

    /// Inserts a parameter value, replacing any existing value for the parameter name.
    ///
    /// Both the key and value are converted to `String` internally, so this method
    /// accepts any types that implement `Into<String>`, including `&str`, `String`,
    /// and other string-like types.
    ///
    /// # Parameters
    ///
    /// - `key`: The parameter name (converted to String)
    /// - `value`: The parameter value (converted to String)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::route_params::RouteParams;
    ///
    /// let mut params = RouteParams::new();
    /// params.insert("id", "123");
    /// params.insert("user_id", 456.to_string()); // Different string types work
    /// params.insert("active".to_string(), "true"); // Owned strings work too
    ///
    /// assert_eq!(params.get("id"), Some("123"));
    /// assert_eq!(params.get("user_id"), Some("456"));
    /// assert_eq!(params.get("active"), Some("true"));
    /// ```
    pub fn insert<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.params.insert(key.into(), value.into());
    }

    /// Returns the string value for the given parameter name, if present.
    ///
    /// This is the fundamental method for parameter access. It returns the raw
    /// string value as captured from the URL. Use parsing methods like
    /// [`get_parsed()`](Self::get_parsed) or [`get_int()`](Self::get_int) to
    /// convert the string to other types.
    ///
    /// # Parameters
    ///
    /// - `name`: The parameter name to look up
    ///
    /// # Returns
    ///
    /// - `Some(&str)` if the parameter exists
    /// - `None` if the parameter is not found
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::route_params::RouteParams;
    ///
    /// let mut params = RouteParams::new();
    /// params.insert("id", "123");
    /// params.insert("slug", "hello-world");
    ///
    /// assert_eq!(params.get("id"), Some("123"));
    /// assert_eq!(params.get("slug"), Some("hello-world"));
    /// assert_eq!(params.get("missing"), None);
    /// ```
    pub fn get(&self, name: &str) -> Option<&str> {
        self.params.get(name).map(|s| s.as_str())
    }

    /// Returns the parameter value parsed into a specific type.
    ///
    /// This method attempts to parse the string parameter into any type that
    /// implements `FromStr`. It provides detailed error information when
    /// parsing fails, including the parameter name, original value, and target type.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The target type to parse into. Must implement `FromStr` and have
    ///   a `Debug` implementation for error reporting.
    ///
    /// # Parameters
    ///
    /// - `name`: The parameter name to retrieve and parse
    ///
    /// # Returns
    ///
    /// - `Ok(T)` if the parameter exists and parses successfully
    /// - `Err(ParamError::NotFound)` if the parameter doesn't exist
    /// - `Err(ParamError::ParseError)` if parsing fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::route_params::RouteParams;
    ///
    /// let mut params = RouteParams::new();
    /// params.insert("id", "123");
    /// params.insert("price", "99.99");
    /// params.insert("active", "true");
    /// params.insert("invalid", "not-a-number");
    ///
    /// // Successful parsing
    /// assert_eq!(params.get_parsed::<i32>("id").unwrap(), 123);
    /// assert_eq!(params.get_parsed::<f64>("price").unwrap(), 99.99);
    /// assert_eq!(params.get_parsed::<bool>("active").unwrap(), true);
    ///
    /// // Missing parameter
    /// assert!(params.get_parsed::<i32>("missing").is_err());
    ///
    /// // Parse error
    /// assert!(params.get_parsed::<i32>("invalid").is_err());
    /// ```
    pub fn get_parsed<T>(&self, name: &str) -> Result<T, RipressError>
    where
        T: FromStr,
        T::Err: std::fmt::Debug,
    {
        let value = self
            .get(name)
            .ok_or_else(|| RipressError::from(ParamError::NotFound(name.to_string())))?;

        value.parse::<T>().map_err(|_| {
            RipressError::from(ParamError::ParseError {
                param: name.to_string(),
                value: value.to_string(),
                target_type: std::any::type_name::<T>().to_string(),
            })
        })
    }

    /// Retrieves a parameter as a signed 32-bit integer.
    ///
    /// This is a convenience method equivalent to `get_parsed::<i32>(name)`.
    /// It's commonly used for database IDs and other integer identifiers.
    ///
    /// # Parameters
    ///
    /// - `name`: The parameter name to retrieve and parse as an integer
    ///
    /// # Returns
    ///
    /// - `Ok(i32)` if the parameter exists and is a valid integer
    /// - `Err(ParamError)` if the parameter is missing or invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::route_params::RouteParams;
    ///
    /// let mut params = RouteParams::new();
    /// params.insert("user_id", "42");
    /// params.insert("negative", "-10");
    ///
    /// assert_eq!(params.get_int("user_id").unwrap(), 42);
    /// assert_eq!(params.get_int("negative").unwrap(), -10);
    /// ```
    pub fn get_int(&self, name: &str) -> Result<i32, RipressError> {
        self.get_parsed::<i32>(name)
    }

    /// Retrieves a parameter as an unsigned 32-bit integer.
    ///
    /// This is a convenience method equivalent to `get_parsed::<u32>(name)`.
    /// It's useful for parameters that should always be positive, like page numbers
    /// or counts.
    ///
    /// # Parameters
    ///
    /// - `name`: The parameter name to retrieve and parse as an unsigned integer
    ///
    /// # Returns
    ///
    /// - `Ok(u32)` if the parameter exists and is a valid positive integer
    /// - `Err(ParamError)` if the parameter is missing, negative, or invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::route_params::RouteParams;
    ///
    /// let mut params = RouteParams::new();
    /// params.insert("page", "5");
    /// params.insert("limit", "20");
    ///
    /// assert_eq!(params.get_uint("page").unwrap(), 5);
    /// assert_eq!(params.get_uint("limit").unwrap(), 20);
    ///
    /// // Negative numbers will fail
    /// params.insert("negative", "-5");
    /// assert!(params.get_uint("negative").is_err());
    /// ```
    pub fn get_uint(&self, name: &str) -> Result<u32, RipressError> {
        self.get_parsed::<u32>(name)
    }

    /// Retrieves a parameter with a fallback default value.
    ///
    /// This method returns the default value if either:
    /// - The parameter doesn't exist
    /// - The parameter exists but cannot be parsed into the target type
    ///
    /// This is useful for optional parameters where you want a sensible fallback
    /// regardless of whether the parameter is missing or malformed.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The target type to parse into, which also determines the default value type
    ///
    /// # Parameters
    ///
    /// - `name`: The parameter name to retrieve and parse
    /// - `default`: The default value to return on missing parameter or parse failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::route_params::RouteParams;
    ///
    /// let mut params = RouteParams::new();
    /// params.insert("page", "3");
    /// params.insert("invalid_limit", "not-a-number");
    ///
    /// // Existing valid parameter
    /// assert_eq!(params.get_or_default("page", 1), 3);
    ///
    /// // Missing parameter - uses default
    /// assert_eq!(params.get_or_default("missing", 1), 1);
    ///
    /// // Invalid parameter - uses default
    /// assert_eq!(params.get_or_default("invalid_limit", 10), 10);
    /// ```
    pub fn get_or_default<T>(&self, name: &str, default: T) -> T
    where
        T: FromStr,
        T::Err: std::fmt::Debug,
    {
        self.get_parsed(name).unwrap_or(default)
    }

    /// Retrieves a parameter, using default only if parsing fails.
    ///
    /// Unlike [`get_or_default()`](Self::get_or_default), this method distinguishes
    /// between missing parameters (which return an error) and parsing failures
    /// (which use the default value). This is useful when you want to ensure
    /// required parameters are present but allow for parsing fallbacks.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The target type to parse into
    ///
    /// # Parameters
    ///
    /// - `name`: The parameter name to retrieve and parse
    /// - `default`: The default value to use if parsing fails
    ///
    /// # Returns
    ///
    /// - `Ok(T)` with the parsed value if successful
    /// - `Ok(T)` with the default if parsing failed but parameter exists
    /// - `Err(ParamError::NotFound)` if the parameter doesn't exist
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::route_params::{RouteParams};
    ///
    /// let mut params = RouteParams::new();
    /// params.insert("page", "5");
    /// params.insert("invalid_limit", "not-a-number");
    ///
    /// // Valid parameter
    /// assert_eq!(params.get_or_parse_default("page", 1).unwrap(), 5);
    ///
    /// // Invalid parameter - uses default
    /// assert_eq!(params.get_or_parse_default("invalid_limit", 10).unwrap(), 10);
    ///
    /// // Missing parameter - returns error
    /// assert!(params.get_or_parse_default("missing", 1).is_err());
    /// ```
    pub fn get_or_parse_default<T>(&self, name: &str, default: T) -> Result<T, RipressError>
    where
        T: FromStr,
        T::Err: std::fmt::Debug,
    {
        match self.get(name) {
            None => Err(RipressError::from(ParamError::NotFound(name.to_string()))),
            Some(value) => Ok(value.parse().unwrap_or(default)),
        }
    }

    /// Checks if a parameter exists by name.
    ///
    /// This method only checks for the presence of the parameter, not whether
    /// its value is valid or can be parsed into specific types.
    ///
    /// # Parameters
    ///
    /// - `name`: The parameter name to check for
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::route_params::RouteParams;
    ///
    /// let mut params = RouteParams::new();
    /// params.insert("id", "123");
    /// params.insert("empty", "");
    ///
    /// assert!(params.contains("id"));
    /// assert!(params.contains("empty")); // Empty values still count as present
    /// assert!(!params.contains("missing"));
    /// ```
    pub fn contains(&self, name: &str) -> bool {
        self.params.contains_key(name)
    }

    /// Returns an iterator over all parameter names.
    ///
    /// This is useful for inspecting what parameters are available, debugging,
    /// or implementing generic parameter processing logic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::route_params::RouteParams;
    ///
    /// let mut params = RouteParams::new();
    /// params.insert("id", "123");
    /// params.insert("slug", "hello-world");
    /// params.insert("format", "json");
    ///
    /// let names: Vec<&String> = params.names().collect();
    /// assert_eq!(names.len(), 3);
    ///
    /// // Note: HashMap iteration order is not guaranteed
    /// for name in params.names() {
    ///     println!("Parameter: {}", name);
    /// }
    /// ```
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.params.keys()
    }

    /// Returns the total number of parameters.
    ///
    /// This counts the number of unique parameter names stored in the collection.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::route_params::RouteParams;
    ///
    /// let mut params = RouteParams::new();
    /// assert_eq!(params.len(), 0);
    ///
    /// params.insert("id", "123");
    /// params.insert("slug", "hello");
    /// assert_eq!(params.len(), 2);
    ///
    /// // Updating existing parameter doesn't change count
    /// params.insert("id", "456");
    /// assert_eq!(params.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.params.len()
    }

    /// Returns `true` if there are no parameters.
    ///
    /// This is equivalent to `len() == 0` but may be more expressive in some contexts.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::route_params::RouteParams;
    ///
    /// let mut params = RouteParams::new();
    /// assert!(params.is_empty());
    ///
    /// params.insert("id", "123");
    /// assert!(!params.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    /// Returns an iterator over all parameter name-value pairs.
    ///
    /// Each item in the iterator is a tuple of `(&String, &String)` representing
    /// the parameter name and its string value. This is useful for generic
    /// processing, debugging, or converting to other formats.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::route_params::RouteParams;
    ///
    /// let mut params = RouteParams::new();
    /// params.insert("user_id", "42");
    /// params.insert("format", "json");
    ///
    /// for (name, value) in params.iter() {
    ///     println!("{} = {}", name, value);
    /// }
    ///
    /// // Collect into a vector for processing
    /// let pairs: Vec<(&String, &String)> = params.iter().collect();
    /// assert_eq!(pairs.len(), 2);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.params.iter()
    }

    /// Consumes the RouteParams and returns the inner HashMap.
    ///
    /// This method transfers ownership of the internal parameter storage,
    /// which can be useful for interoperability with other libraries or
    /// when you need direct HashMap operations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::route_params::RouteParams;
    /// use std::collections::HashMap;
    ///
    /// let mut params = RouteParams::new();
    /// params.insert("id", "123");
    /// params.insert("slug", "hello-world");
    ///
    /// let map: HashMap<String, String> = params.into_map();
    /// assert_eq!(map.get("id"), Some(&"123".to_string()));
    /// assert_eq!(map.get("slug"), Some(&"hello-world".to_string()));
    ///
    /// // params is no longer accessible after this operation
    /// ```
    pub fn into_map(self) -> HashMap<String, String> {
        self.params
    }

    /// Runs a custom validation function over the parameters.
    ///
    /// This method provides a way to implement complex validation logic that
    /// might involve multiple parameters or custom business rules. The extractor
    /// function receives a reference to the `RouteParams` and should return
    /// either `Ok(())` for success or `Err(Vec<RipressError>)` for failures.
    ///
    /// # Parameters
    ///
    /// - `extractor`: A closure that takes `&Self` and returns a validation result
    ///
    /// # Returns
    ///
    /// - `Ok(())` if validation passes
    /// - `Err(Vec<RipressError>)` containing all validation errors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::route_params::{RouteParams};
    ///
    /// let mut params = RouteParams::new();
    /// params.insert("user_id", "42");
    /// params.insert("post_id", "123");
    ///
    /// // Validate that both required parameters exist and are valid integers
    /// let result = params.extract(|p| {
    ///     let mut errors = Vec::new();
    ///     
    ///     if let Err(e) = p.get_int("user_id") {
    ///         errors.push(e);
    ///     }
    ///     if let Err(e) = p.get_int("post_id") {
    ///         errors.push(e);
    ///     }
    ///     
    ///     if errors.is_empty() {
    ///         Ok(())
    ///     } else {
    ///         Err(errors)
    ///     }
    /// });
    ///
    /// assert!(result.is_ok());
    /// ```
    pub fn extract<F>(&self, extractor: F) -> Result<(), Vec<RipressError>>
    where
        F: FnOnce(&Self) -> Result<(), Vec<RipressError>>,
    {
        extractor(self)
    }

    // --- Convenience Methods ---

    /// Retrieves the `id` parameter as an integer.
    ///
    /// This is a convenience method for the very common case of having an `id`
    /// parameter in routes like `/users/:id` or `/posts/:id`. It's equivalent
    /// to calling `get_int("id")`.
    ///
    /// # Returns
    ///
    /// - `Ok(i32)` if the `id` parameter exists and is a valid integer
    /// - `Err(ParamError)` if the parameter is missing or invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::route_params::RouteParams;
    ///
    /// let mut params = RouteParams::new();
    /// params.insert("id", "42");
    ///
    /// assert_eq!(params.id().unwrap(), 42);
    ///
    /// // Equivalent to:
    /// assert_eq!(params.get_int("id").unwrap(), 42);
    /// ```
    pub fn id(&self) -> Result<i32, RipressError> {
        self.get_int("id")
    }

    /// Retrieves the `slug` parameter as a string slice.
    ///
    /// This is a convenience method for routes that use URL-friendly string
    /// identifiers, commonly called "slugs". Examples include blog post slugs
    /// like "hello-world" or product slugs like "red-widget".
    ///
    /// # Returns
    ///
    /// - `Some(&str)` if the `slug` parameter exists
    /// - `None` if the `slug` parameter is missing
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::route_params::RouteParams;
    ///
    /// let mut params = RouteParams::new();
    /// params.insert("slug", "hello-world");
    ///
    /// assert_eq!(params.slug(), Some("hello-world"));
    ///
    /// // Equivalent to:
    /// assert_eq!(params.get("slug"), Some("hello-world"));
    /// ```
    pub fn slug(&self) -> Option<&str> {
        self.get("slug")
    }
}

impl Default for RouteParams {
    /// Creates an empty RouteParams instance.
    ///
    /// This implementation allows `RouteParams` to be used with `Default::default()`
    /// and in contexts where a default instance is needed.
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for RouteParams {
    /// Formats the parameters as comma-separated `key=value` pairs.
    ///
    /// This provides a human-readable representation of all parameters,
    /// useful for debugging, logging, or displaying parameter information.
    ///
    /// # Format
    ///
    /// Parameters are formatted as `name=value` and separated by commas and spaces.
    /// Empty parameter collections display as an empty string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::route_params::RouteParams;
    ///
    /// let mut params = RouteParams::new();
    /// params.insert("user_id", "42");
    /// params.insert("format", "json");
    ///
    /// let display = format!("{}", params);
    /// // Output might be: "user_id=42, format=json" (order not guaranteed)
    /// println!("Parameters: {}", params);
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let param_strings: Vec<String> = self
            .params
            .iter()
            .map(|(name, value)| format!("{}={}", name, value))
            .collect();
        write!(f, "{}", param_strings.join(", "))
    }
}

impl std::ops::Index<&str> for RouteParams {
    type Output = str;

    /// Provides convenient indexing syntax:
    ///
    /// ```
    /// use ripress::req::route_params::RouteParams;
    ///
    /// let mut params = RouteParams::new();
    /// params.insert("user_id", "something");
    /// assert_eq!(&params["user_id"], "something");
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the param does not exist.

    fn index(&self, name: &str) -> &Self::Output {
        self.get(name)
            .unwrap_or_else(|| panic!("Route parameter '{}' not found", name))
    }
}

// Convert from HashMap for easy migration
impl From<HashMap<String, String>> for RouteParams {
    fn from(map: HashMap<String, String>) -> Self {
        Self::from_map(map)
    }
}

// Convert to HashMap for backward compatibility
impl From<RouteParams> for HashMap<String, String> {
    fn from(params: RouteParams) -> Self {
        params.params
    }
}
