use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

/// Route parameters extracted from URL patterns like `/users/:id` or `/posts/:slug/comments/:comment_id`
#[derive(Debug, Clone)]
pub struct RouteParams {
    /// Internal storage of parameter name -> value mappings
    params: HashMap<String, String>,
}

/// Error type for parameter parsing failures
#[derive(Debug, Clone, PartialEq)]
pub enum ParamError {
    /// Parameter not found
    NotFound(String),
    /// Parameter exists but couldn't be parsed to the requested type
    ParseError {
        param: String,
        value: String,
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
    /// Create a new empty RouteParams
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
        }
    }

    /// Create RouteParams from a HashMap (for backward compatibility)
    pub fn from_map(map: HashMap<String, String>) -> Self {
        Self { params: map }
    }

    /// Insert a parameter
    pub fn insert<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.params.insert(key.into(), value.into());
    }

    /// Get a parameter value as a string
    pub fn get(&self, name: &str) -> Option<&str> {
        self.params.get(name).map(|s| s.as_str())
    }

    /// Get a parameter and parse it to a specific type
    pub fn get_parsed<T>(&self, name: &str) -> Result<T, ParamError>
    where
        T: FromStr,
        T::Err: std::fmt::Debug,
    {
        let value = self
            .get(name)
            .ok_or_else(|| ParamError::NotFound(name.to_string()))?;

        value.parse::<T>().map_err(|_| ParamError::ParseError {
            param: name.to_string(),
            value: value.to_string(),
            target_type: std::any::type_name::<T>().to_string(),
        })
    }

    /// Get a parameter as an integer (common case)
    pub fn get_int(&self, name: &str) -> Result<i32, ParamError> {
        self.get_parsed::<i32>(name)
    }

    /// Get a parameter as an unsigned integer
    pub fn get_uint(&self, name: &str) -> Result<u32, ParamError> {
        self.get_parsed::<u32>(name)
    }

    /// Get a parameter as a UUID (if using uuid crate)
    #[cfg(feature = "uuid")]
    pub fn get_uuid(&self, name: &str) -> Result<uuid::Uuid, ParamError> {
        self.get_parsed::<uuid::Uuid>(name)
    }

    /// Get a parameter with a default value if not found or parsing fails
    pub fn get_or_default<T>(&self, name: &str, default: T) -> T
    where
        T: FromStr,
        T::Err: std::fmt::Debug,
    {
        self.get_parsed(name).unwrap_or(default)
    }

    /// Get a parameter with a default value, but only if parsing fails (not if missing)
    pub fn get_or_parse_default<T>(&self, name: &str, default: T) -> Result<T, ParamError>
    where
        T: FromStr,
        T::Err: std::fmt::Debug,
    {
        match self.get(name) {
            None => Err(ParamError::NotFound(name.to_string())),
            Some(value) => Ok(value.parse().unwrap_or(default)),
        }
    }

    /// Check if a parameter exists
    pub fn contains(&self, name: &str) -> bool {
        self.params.contains_key(name)
    }

    /// Get all parameter names
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.params.keys()
    }

    /// Get the number of parameters
    pub fn len(&self) -> usize {
        self.params.len()
    }

    /// Check if there are no parameters
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    /// Iterate over all parameters as (name, value) pairs
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.params.iter()
    }

    /// Convert back to HashMap (for backward compatibility)
    pub fn into_map(self) -> HashMap<String, String> {
        self.params
    }

    /// Extract multiple parameters at once with validation
    /// Returns Ok(()) if all parameters exist and parse successfully
    pub fn extract<F>(&self, extractor: F) -> Result<(), Vec<ParamError>>
    where
        F: FnOnce(&Self) -> Result<(), Vec<ParamError>>,
    {
        extractor(self)
    }

    // Common parameter name patterns

    /// Get 'id' parameter as integer (very common pattern)
    pub fn id(&self) -> Result<i32, ParamError> {
        self.get_int("id")
    }

    /// Get 'slug' parameter (common for SEO-friendly URLs)
    pub fn slug(&self) -> Option<&str> {
        self.get("slug")
    }
}

impl Default for RouteParams {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for RouteParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let param_strings: Vec<String> = self
            .params
            .iter()
            .map(|(name, value)| format!("{}={}", name, value))
            .collect();
        write!(f, "{}", param_strings.join(", "))
    }
}

// Convenient indexing syntax: params["id"]
impl std::ops::Index<&str> for RouteParams {
    type Output = str;

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

/// Macro for easy parameter extraction with validation
#[macro_export]
macro_rules! extract_params {
    ($params:expr, { $($name:ident: $type:ty),* $(,)? }) => {{
        let mut errors = Vec::new();
        $(
            let $name = match $params.get_parsed::<$type>(stringify!($name)) {
                Ok(val) => val,
                Err(e) => {
                    errors.push(e);
                    continue;
                }
            };
        )*

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(($($name,)*))
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut params = RouteParams::new();
        params.insert("id", "123");
        params.insert("slug", "hello-world");

        assert_eq!(params.get("id"), Some("123"));
        assert_eq!(params.get("slug"), Some("hello-world"));
        assert_eq!(params.get("missing"), None);
    }

    #[test]
    fn test_type_parsing() {
        let mut params = RouteParams::new();
        params.insert("id", "123");
        params.insert("user_id", "456");
        params.insert("invalid", "not-a-number");

        assert_eq!(params.get_int("id").unwrap(), 123);
        assert!(params.get_int("invalid").is_err());
        assert!(params.get_int("missing").is_err());
    }

    #[test]
    fn test_convenience_methods() {
        let mut params = RouteParams::new();
        params.insert("id", "42");
        params.insert("slug", "test-post");
        params.insert("user_id", "100");

        assert_eq!(params.id().unwrap(), 42);
        assert_eq!(params.slug(), Some("test-post"));
    }

    #[test]
    fn test_error_types() {
        let params = RouteParams::new();

        match params.get_int("missing") {
            Err(ParamError::NotFound(name)) => assert_eq!(name, "missing"),
            _ => panic!("Expected NotFound error"),
        }

        let mut params = RouteParams::new();
        params.insert("invalid", "not-a-number");

        match params.get_int("invalid") {
            Err(ParamError::ParseError { param, value, .. }) => {
                assert_eq!(param, "invalid");
                assert_eq!(value, "not-a-number");
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_defaults() {
        let mut params = RouteParams::new();
        params.insert("valid", "10");

        assert_eq!(params.get_or_default("valid", 5), 10);
        assert_eq!(params.get_or_default("missing", 5), 5);
        assert_eq!(params.get_or_default("invalid", 5), 5);
    }

    #[test]
    fn test_from_map() {
        let mut map = HashMap::new();
        map.insert("id".to_string(), "123".to_string());
        map.insert("name".to_string(), "test".to_string());

        let params = RouteParams::from_map(map);
        assert_eq!(params.get("id"), Some("123"));
        assert_eq!(params.get("name"), Some("test"));
    }

    // Example of using the extract_params macro
    #[test]
    fn test_extract_macro() {
        let mut params = RouteParams::new();
        params.insert("id", "123");
        params.insert("user_id", "456");

        // This would be used in a real handler like:
        // let (id, user_id) = extract_params!(params, { id: i32, user_id: i32 })?;
    }
}
