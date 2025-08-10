use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

/// A collection of parameters extracted from a route's URL pattern.
///
/// `RouteParams` is typically used to store values captured from
/// path parameters in routes like:
///
/// - `/users/:id`
/// - `/posts/:slug/comments/:comment_id`
///
/// The parameters are stored as a map from their **name** to their **string value**,
/// and can be retrieved as strings or parsed into other types.
///
/// ## Example
///
/// ```
/// use ripress::req::route_params::RouteParams;
///
/// let mut params = RouteParams::new();
/// params.insert("id", "42");
///
/// assert_eq!(params.get("id"), Some("42"));
/// assert_eq!(params.get_int("id").unwrap(), 42);
/// ```

#[derive(Debug, Clone)]
pub struct RouteParams {
    /// Internal storage mapping parameter names to their values.
    params: HashMap<String, String>,
}

/// Errors that can occur when retrieving parameters.
#[derive(Debug, Clone, PartialEq)]
pub enum ParamError {
    /// The parameter with the given name does not exist.
    NotFound(String),

    /// The parameter exists but could not be parsed into the requested type.
    ParseError {
        /// The parameter name.
        param: String,

        /// The original string value.
        value: String,

        /// The target type's name.
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
    /// # Example
    /// ```
    /// use ripress::req::route_params::RouteParams;
    ///
    /// let params = RouteParams::new();
    /// assert!(params.is_empty());
    /// ```

    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
        }
    }

    /// Creates a `RouteParams` instance from an existing `HashMap`.
    ///
    /// # Example
    /// ```
    /// use ripress::req::route_params::RouteParams;
    /// use std::collections::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("id".into(), "123".into());
    /// let params = RouteParams::from_map(map);
    /// assert_eq!(params.get("id"), Some("123"));
    /// ```

    pub fn from_map(map: HashMap<String, String>) -> Self {
        Self { params: map }
    }

    /// Inserts a param value, replacing any existing values for the param name.
    ///
    /// # Example
    /// ```
    /// use ripress::req::route_params::RouteParams;
    ///
    /// let mut param = RouteParams::new();
    /// param.insert("id", "123");
    /// assert_eq!(param.get("id"), Some("123"));
    /// ```

    pub fn insert<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.params.insert(key.into(), value.into());
    }

    /// Returns the  value for the given param name, if present.
    ///
    /// # Example
    /// ```
    /// use ripress::req::route_params::RouteParams;
    ///
    /// let mut params = RouteParams::new();
    /// params.insert("id", "123");
    /// assert_eq!(params.get("id"), Some("123"));
    /// assert_eq!(params.get("missing"), None);
    /// ```

    pub fn get(&self, name: &str) -> Option<&str> {
        self.params.get(name).map(|s| s.as_str())
    }

    /// Returns the  value for the given param name, parsed into a specific type.
    ///
    /// # Example
    /// ```
    /// use ripress::req::route_params::RouteParams;
    ///
    /// let mut params = RouteParams::new();
    /// params.insert("id", "123");
    /// assert_eq!(params.get_parsed::<i32>("id"), Ok(123));
    /// ```
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

    /// Retrieves a parameter as an `i32`.
    pub fn get_int(&self, name: &str) -> Result<i32, ParamError> {
        self.get_parsed::<i32>(name)
    }

    /// Retrieves a parameter as a `u32`.
    pub fn get_uint(&self, name: &str) -> Result<u32, ParamError> {
        self.get_parsed::<u32>(name)
    }

    /// Retrieves a parameter with a default if missing or parsing fails.
    pub fn get_or_default<T>(&self, name: &str, default: T) -> T
    where
        T: FromStr,
        T::Err: std::fmt::Debug,
    {
        self.get_parsed(name).unwrap_or(default)
    }

    /// Retrieves a parameter, returning the default only if parsing fails.
    ///
    /// Missing parameters still result in an error.
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

    /// Checks if a parameter exists by name.
    pub fn contains(&self, name: &str) -> bool {
        self.params.contains_key(name)
    }

    /// Returns an iterator over all parameter names.
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.params.keys()
    }

    /// Returns the total number of parameters.
    pub fn len(&self) -> usize {
        self.params.len()
    }

    /// Returns `true` if there are no parameters.
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    /// Iterates over all `(name, value)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.params.iter()
    }

    /// Consumes the struct and returns the inner `HashMap`.
    pub fn into_map(self) -> HashMap<String, String> {
        self.params
    }

    /// Runs a custom extraction function over the parameters.
    ///
    /// Useful for validating the presence and type of multiple parameters at once.
    pub fn extract<F>(&self, extractor: F) -> Result<(), Vec<ParamError>>
    where
        F: FnOnce(&Self) -> Result<(), Vec<ParamError>>,
    {
        extractor(self)
    }

    // --- Convenience Methods ---

    /// Retrieves the `id` parameter as an integer.
    pub fn id(&self) -> Result<i32, ParamError> {
        self.get_int("id")
    }

    /// Retrieves the `slug` parameter as a string slice.
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
    /// Formats the params as `key: value` lines.
    ///
    /// # Example
    /// ```
    /// use ripress::req::route_params::RouteParams;
    ///
    /// let mut params = RouteParams::new();
    /// params.insert("user_id", "something");
    /// println!("{}", params);
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
