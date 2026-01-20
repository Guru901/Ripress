#![warn(missing_docs)]
use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

use ahash::AHashMap;

use crate::error::RipressError;
use crate::helpers::FromRequest;

/// Query parameters from URL query string with support for multiple values
/// Handles URLs like: `/search?q=rust&tags=web&tags=backend&page=1&active=true`
#[derive(Debug, Clone)]
pub struct QueryParams {
    /// Internal storage: parameter name -> list of values
    /// Supports multiple values for the same parameter (e.g., multiple tags)
    pub(crate) inner: AHashMap<String, Vec<String>>,
}

/// Error type for query parameter parsing and retrieval failures.
///
/// This enum represents the different ways that query parameter operations can fail,
/// providing detailed context about what went wrong to help with debugging and error handling.
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use ripress::req::query_params::QueryParams;
///
/// let params = QueryParams::from_query_string("page=abc&tags=rust&tags=web");
///
/// // Parameter not found
/// assert!(params.get_int("missing").is_err());
///
/// // Parse error
/// assert!(params.get_int("page").is_err());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum QueryParamError {
    /// The requested query parameter was not found.
    ///
    /// This error occurs when attempting to retrieve a parameter that doesn't exist
    /// in the query string. The contained `String` is the name of the missing parameter.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::query_params::QueryParams;
    ///
    /// let params = QueryParams::from_query_string("page=1&limit=20");
    /// assert!(params.get_int("offset").is_err());
    /// ```
    NotFound(String),

    /// The parameter exists but could not be parsed to the requested type.
    ///
    /// This error occurs when a parameter is present in the query string but its value
    /// cannot be converted to the target type (e.g., trying to parse "abc" as an integer).
    ///
    /// # Fields
    ///
    /// * `param` - The name of the parameter that failed to parse
    /// * `value` - The actual string value that couldn't be converted
    /// * `target_type` - The type that parsing was attempted for (from `std::any::type_name`)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::query_params::QueryParams;
    ///
    /// let params = QueryParams::from_query_string("page=not_a_number&active=maybe");
    ///
    /// // Trying to parse non-numeric string as integer
    /// assert!(params.get_int("page").is_err());
    ///
    /// // Invalid boolean value
    /// assert!(params.get_bool("active").is_err());
    /// ```
    ParseError {
        /// The name of the parameter that failed to parse
        param: String,
        /// The actual string value that couldn't be converted
        value: String,
        /// The target type that parsing was attempted for
        target_type: String,
    },
}

impl fmt::Display for QueryParamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryParamError::NotFound(param) => write!(f, "Query parameter '{}' not found", param),
            QueryParamError::ParseError {
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

impl std::error::Error for QueryParamError {}

impl QueryParams {
    /// Create a new empty QueryParams
    pub fn new() -> Self {
        Self {
            inner: AHashMap::new(),
        }
    }

    /// Create QueryParams from a single-value AHashMap
    pub fn from_ahashmap(map: AHashMap<String, String>) -> Self {
        let mut params = AHashMap::with_capacity(map.len());
        for (key, value) in map {
            params.insert(key, vec![value]);
        }
        Self { inner: params }
    }

    /// Create QueryParams from a single-value HashMap (for backward compatibility)
    pub fn from_map(map: HashMap<String, String>) -> Self {
        let mut params = AHashMap::with_capacity(map.len());
        for (key, value) in map {
            params.insert(key, vec![value]);
        }
        Self { inner: params }
    }

    /// Create QueryParams from an iterator of key-value pairs
    pub fn from_iterator(iterator: impl Iterator<Item = (String, String)>) -> Self {
        let mut params = AHashMap::new();
        for (key, value) in iterator {
            params.insert(key, vec![value]);
        }
        Self { inner: params }
    }

    /// Parse query parameters from a query string
    /// Example: "q=rust&tags=web&tags=backend&page=1&active=true"
    pub fn from_query_string(query_string: &str) -> Self {
        let mut params = AHashMap::new();

        if query_string.is_empty() {
            return Self { inner: params };
        }

        for pair in query_string.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                let key = urlencoding::decode(key)
                    .unwrap_or_else(|_| key.into())
                    .into_owned();
                let value = urlencoding::decode(value)
                    .unwrap_or_else(|_| value.into())
                    .into_owned();

                params.entry(key).or_insert_with(Vec::new).push(value);
            } else if !pair.is_empty() {
                let key = urlencoding::decode(pair)
                    .unwrap_or_else(|_| pair.into())
                    .into_owned();
                params
                    .entry(key)
                    .or_insert_with(Vec::new)
                    .push(String::new());
            }
        }

        Self { inner: params }
    }

    /// Insert a single parameter value (replaces existing)
    pub fn insert<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.inner
            .entry(key.into())
            .or_insert_with(Vec::new)
            .push(value.into());
    }

    /// Append a parameter value (supports multiple values)
    pub fn append<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.inner.entry(key.into()).or_default().push(value.into());
    }

    /// Get the first value for a parameter (most common case)
    pub fn get(&self, name: &str) -> Option<&str> {
        self.inner.get(name)?.first().map(|s| s.as_str())
    }

    /// Get all values for a parameter
    pub fn get_all(&self, name: &str) -> Option<&Vec<String>> {
        self.inner.get(name)
    }

    /// Get the first value and parse it to a specific type
    pub fn get_parsed<T>(&self, name: &str) -> Result<T, RipressError>
    where
        T: FromStr,
        T::Err: std::fmt::Debug,
    {
        let value = self
            .get(name)
            .ok_or_else(|| QueryParamError::NotFound(name.to_string()))?;

        value.parse::<T>().map_err(|_| {
            RipressError::from(QueryParamError::ParseError {
                param: name.to_string(),
                value: value.to_string(),
                target_type: std::any::type_name::<T>().to_string(),
            })
        })
    }

    /// Get all values and parse them to a specific type
    pub fn get_all_parsed<T>(&self, name: &str) -> Result<Vec<T>, RipressError>
    where
        T: FromStr,
        T::Err: std::fmt::Debug,
    {
        let values = self
            .get_all(name)
            .ok_or_else(|| QueryParamError::NotFound(name.to_string()))?;

        let mut parsed_values = Vec::new();
        for value in values {
            let parsed = value
                .parse::<T>()
                .map_err(|_| QueryParamError::ParseError {
                    param: name.to_string(),
                    value: value.clone(),
                    target_type: std::any::type_name::<T>().to_string(),
                })?;
            parsed_values.push(parsed);
        }

        Ok(parsed_values)
    }

    /// Get parameter as integer
    pub fn get_int(&self, name: &str) -> Result<i32, RipressError> {
        self.get_parsed::<i32>(name)
    }

    /// Get parameter as 64-bit integer
    pub fn get_i64(&self, name: &str) -> Result<i64, RipressError> {
        self.get_parsed::<i64>(name)
    }

    /// Get parameter as unsigned integer
    pub fn get_uint(&self, name: &str) -> Result<u32, RipressError> {
        self.get_parsed::<u32>(name)
    }

    /// Get parameter as boolean
    /// Supports: "true"/"false", "1"/"0", "yes"/"no", "on"/"off"
    pub fn get_bool(&self, name: &str) -> Result<bool, RipressError> {
        let value = self
            .get(name)
            .ok_or_else(|| QueryParamError::NotFound(name.to_string()))?;

        match value.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Ok(true),
            "false" | "0" | "no" | "off" | "" => Ok(false),
            _ => Err(RipressError::from(QueryParamError::ParseError {
                param: name.to_string(),
                value: value.to_string(),
                target_type: "bool".to_string(),
            })),
        }
    }

    /// Get parameter as float
    pub fn get_float(&self, name: &str) -> Result<f64, RipressError> {
        self.get_parsed::<f64>(name)
    }

    /// Get parameter with a default value if not found or parsing fails
    pub fn get_or_default<T>(&self, name: &str, default: T) -> T
    where
        T: FromStr,
        T::Err: std::fmt::Debug,
    {
        self.get_parsed(name).unwrap_or(default)
    }

    /// Check if a parameter exists (even with empty value)
    pub fn contains(&self, name: &str) -> bool {
        self.inner.contains_key(name)
    }

    /// Check if a parameter has a non-empty value
    pub fn has_value(&self, name: &str) -> bool {
        self.get(name).map_or(false, |v| !v.is_empty())
    }

    /// Get all parameter names
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.inner.keys()
    }

    /// Get the number of unique parameters
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if there are no parameters
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Iterate over all parameters as (name, first_value) pairs
    pub fn iter(&self) -> impl Iterator<Item = (&String, &str)> {
        self.inner
            .iter()
            .filter_map(|(k, v)| v.first().map(|first_val| (k, first_val.as_str())))
    }

    /// Iterate over all parameters including multiple values
    pub fn iter_all(&self) -> impl Iterator<Item = (&String, &Vec<String>)> {
        self.inner.iter()
    }

    /// Convert to single-value HashMap (takes first value for each param)
    pub fn into_map(self) -> HashMap<String, String> {
        self.inner
            .into_iter()
            .filter_map(|(k, mut v)| v.pop().map(|val| (k, val)))
            .collect()
    }

    /// Remove a parameter completely
    pub fn remove(&mut self, name: &str) -> Option<Vec<String>> {
        self.inner.remove(name)
    }


    /// Get 'page' parameter (pagination)
    pub fn page(&self) -> i32 {
        self.get_or_default("page", 1)
    }

    /// Get 'limit' or 'per_page' parameter (pagination)
    pub fn limit(&self) -> i32 {
        self.get_or_default("limit", 20)
            .max(self.get_or_default("per_page", 20))
    }

    /// Get 'q' or 'query' or 'search' parameter (search)
    pub fn search_query(&self) -> Option<&str> {
        self.get("q")
            .or_else(|| self.get("query"))
            .or_else(|| self.get("search"))
    }

    /// Get 'sort' or 'order_by' parameter (sorting)
    pub fn sort(&self) -> Option<&str> {
        self.get("sort").or_else(|| self.get("order_by"))
    }

    /// Get 'order' or 'dir' or 'direction' parameter (sort direction)
    pub fn sort_direction(&self) -> SortDirection {
        let value = self
            .get("order")
            .or_else(|| self.get("dir"))
            .or_else(|| self.get("direction"))
            .unwrap_or("asc");

        match value.to_lowercase().as_str() {
            "desc" | "descending" | "down" => SortDirection::Desc,
            _ => SortDirection::Asc,
        }
    }

    /// Get 'offset' parameter (pagination alternative)
    pub fn offset(&self) -> i32 {
        self.get_or_default("offset", 0)
    }

    /// Get filter parameters (common pattern: filter[status]=active&filter[type]=user)
    pub fn filters(&self) -> HashMap<String, Vec<String>> {
        let mut filters = HashMap::new();

        for (key, values) in &self.inner {
            if let Some(filter_key) = key
                .strip_prefix("filter[")
                .and_then(|k| k.strip_suffix("]"))
            {
                filters.insert(filter_key.to_string(), values.clone());
            }
        }

        filters
    }

    /// Check if parameter indicates "true" value (flexible boolean parsing)
    pub fn is_truthy(&self, name: &str) -> bool {
        self.get_bool(name).unwrap_or(false) || self.contains(name)
    }
}

/// Sort direction enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    /// Ascending sort
    Asc,

    /// Descending sort
    Desc,
}

impl fmt::Display for SortDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortDirection::Asc => write!(f, "asc"),
            SortDirection::Desc => write!(f, "desc"),
        }
    }
}

impl Default for QueryParams {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for QueryParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let param_strings: Vec<String> = self
            .inner
            .iter()
            .flat_map(|(name, values)| {
                values.iter().map(move |value| {
                    if value.is_empty() {
                        name.clone()
                    } else {
                        format!("{}={}", name, value)
                    }
                })
            })
            .collect();
        write!(f, "{}", param_strings.join("&"))
    }
}

impl std::ops::Index<&str> for QueryParams {
    type Output = str;

    fn index(&self, name: &str) -> &Self::Output {
        self.get(name)
            .unwrap_or_else(|| panic!("Query parameter '{}' not found", name))
    }
}

impl From<AHashMap<String, String>> for QueryParams {
    fn from(map: AHashMap<String, String>) -> Self {
        Self::from_ahashmap(map)
    }
}

impl From<HashMap<String, String>> for QueryParams {
    fn from(map: HashMap<String, String>) -> Self {
        Self::from_map(map)
    }
}

/// Wrapper type for parameter values extracted from query parameters in a request.
///
/// Automatically dereferences to the contained inner value of type `T`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QueryParam<T>(pub T);

impl<T> Deref for QueryParam<T> {
    type Target = T;

    /// Dereferences to the contained value.
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Trait for extracting typed values from query parameters.
///
/// Types that implement this trait can be constructed from a set of query parameters,
/// facilitating type-safe parameter extraction from HTTP requests.
pub trait FromQueryParam: Sized {
    /// Attempt to extract `Self` from `params` (the parsed route parameters).
    ///
    /// # Arguments
    ///
    /// * `params` - Reference to the parsed query parameters.
    ///
    /// # Returns
    ///
    /// `Ok(Self)` if extraction is successful, or `Err(String)` if it fails due to missing or malformed input.
    fn from_query_param(params: &QueryParams) -> Result<Self, String>;
}

impl<T: FromQueryParam> FromRequest for QueryParam<T> {
    type Error = String;

    fn from_request(req: &super::HttpRequest) -> Result<Self, Self::Error> {
        Ok(Self(T::from_query_param(&req.query)?))
    }
}
