use crate::types::{HttpMethods, HttpRequestError, RequestBodyContent, RequestBodyType};
use cookie::Cookie;
use hyper::{body::to_bytes, Body, Request};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct RequestBody {
    content: RequestBodyContent,
    content_type: RequestBodyType,
}

// Represents an incoming HTTP request with comprehensive access to request data.
///
/// The HttpRequest struct provides methods to access and manipulate all aspects
/// of an HTTP request including headers, cookies, query parameters, route parameters,
/// and request body content.
///
/// ## Examples
///
/// Basic usage:
/// ```rust
/// use ripress::context::HttpRequest;
///
/// let req = HttpRequest::new();
/// println!("Method: {:?}", req.get_method());
/// println!("Path: {}", req.get_path());
/// println!("Client IP: {:?}", req.ip());
/// ```
///
/// Working with JSON body:
/// ```rust
/// use ripress::context::HttpRequest;
/// use serde::{Deserialize, Serialize};
///
/// ##[derive(Deserialize, Serialize)]
/// struct User {
///     name: String,
///     age: u32
/// }
///
/// let req = HttpRequest::new();
/// if let Ok(user) = req.json::<User>() {
///     println!("User: {} is {} years old", user.name, user.age);
/// }
/// ```

#[derive(Debug, Clone)]
pub struct HttpRequest {
    /// Dynamic route parameters extracted from the URL.
    params: HashMap<String, String>,

    /// Query parameters from the request URL.
    queries: HashMap<String, String>,

    /// The request body, which may contain JSON, text, or form data.
    body: RequestBody,

    /// The IP address of the client making the request.
    ip: String,

    /// The HTTP method used for the request (e.g., GET, POST, PUT, DELETE).
    method: HttpMethods,

    /// The full URL of the incoming request.
    origin_url: String,

    /// The requested endpoint path.
    path: String,

    /// The request's headers
    headers: HashMap<String, String>,

    /// The request's cookies
    cookies: HashMap<String, String>,

    /// Protocol of the request (HTTP or HTTPs)
    protocol: String,

    data: HashMap<String, String>,
}

impl HttpRequest {
    /// Creates a new empty HTTP request instance.
    ///
    /// ## Returns
    ///
    /// Returns a new `HttpRequest` with default values.
    ///
    /// ## Example
    /// ```rust
    /// use ripress::context::HttpRequest;
    /// use ripress::types::HttpMethods;
    ///
    /// let req = HttpRequest::new();
    /// assert_eq!(req.get_method(), &HttpMethods::GET);
    /// ```
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
            queries: HashMap::new(),
            body: RequestBody {
                content_type: RequestBodyType::TEXT,
                content: RequestBodyContent::TEXT(String::new()),
            },
            ip: String::new(),
            method: HttpMethods::GET,
            origin_url: String::new(),
            path: String::new(),
            headers: HashMap::new(),
            cookies: HashMap::new(),
            protocol: String::from("http"),
            data: HashMap::new(),
        }
    }

    /// Checks if the request body matches a specific content type.
    ///
    /// ## Arguments
    ///
    /// * `content_type` - The `RequestBodyType` to check against
    ///
    /// ## Returns
    ///
    /// Returns `true` if the content type matches, `false` otherwise.
    ///
    /// ## Example
    /// ```rust
    /// use ripress::{context::HttpRequest, types::RequestBodyType};
    ///
    /// let req = HttpRequest::new();
    /// if req.is(RequestBodyType::JSON) {
    ///     // Handle JSON content
    /// }
    /// ```

    pub fn is(&self, content_type: RequestBodyType) -> bool {
        self.body.content_type == content_type
    }

    /// Returns the HTTP method used for this request.
    ///
    /// ## Returns
    ///
    /// Returns a reference to the `HttpMethods` enum representing the request method.
    ///
    /// ## Example
    /// ```rust
    /// use ripress::context::HttpRequest;
    /// use ripress::types::HttpMethods;
    ///
    /// let req = HttpRequest::new();
    /// match req.get_method() {
    ///     HttpMethods::GET => println!("Handling GET request"),
    ///     HttpMethods::POST => println!("Handling POST request"),
    ///     _ => println!("Handling other method")
    /// }
    /// ```

    pub fn get_method(&self) -> &HttpMethods {
        &self.method
    }

    /// Returns the request's origin URL.
    ///
    /// ## Returns
    ///
    /// Returns `Ok(&str)` with the origin url value if found, or
    /// `Err(&str)` if not found.
    ///
    /// ## Example
    /// ```
    /// let req = ripress::context::HttpRequest::new();
    /// req.get_origin_url();
    /// ```

    pub fn get_origin_url(&self) -> Result<&str, &str> {
        let origin_url = &self.origin_url;
        if origin_url.len() != 0 {
            Ok(origin_url)
        } else {
            Err("Error getting origin url")
        }
    }

    /// Retrieves a cookie value by name.
    ///
    /// ## Arguments
    ///
    /// * `name` - The name of the cookie to retrieve
    ///
    /// ## Returns
    ///
    /// Returns `Ok(&str)` with the cookie value if found, or
    /// `Err(HttpRequestError::MissingCookie)` if not found.
    ///
    /// ## Example
    /// ```rust
    /// use ripress::context::HttpRequest;
    ///
    /// let req = HttpRequest::new();
    /// match req.get_cookie("session_id") {
    ///     Ok(session) => println!("Session ID: {}", session),
    ///     Err(e) => println!("No session cookie found: {:?}", e)
    /// }
    /// ```

    pub fn get_cookie(&self, name: &str) -> Result<&str, HttpRequestError> {
        let cookie = self.cookies.get(name);

        match cookie {
            Some(cookie_str) => Ok(cookie_str),
            None => Err(HttpRequestError::MissingCookie(name.to_string())),
        }
    }

    /// Returns the request's path.
    ///
    /// ## Returns
    ///
    /// Returns String with the path
    ///
    /// ## Example
    /// ```
    /// let req = ripress::context::HttpRequest::new();
    /// req.get_path();
    /// ```

    pub fn get_path(&self) -> &str {
        self.path.as_str()
    }

    /// Returns the client's IP address.
    ///
    /// ## Returns
    ///
    /// Returns `Ok(&str)` with the ip value if found, or
    /// `Err(err)` if not found.
    ///
    /// ## Example
    /// ```
    /// let req = ripress::context::HttpRequest::new();
    /// let ip = req.ip();
    /// println!("Client IP: {:?}", ip);
    /// ```

    pub fn ip(&self) -> Result<&str, &str> {
        let ip_str = self.ip.as_str();

        if ip_str.len() != 0 {
            Ok(ip_str)
        } else {
            Err("Cannot determine the ip")
        }
    }

    /// Returns url parameters.
    ///
    /// ## Arguments
    ///
    /// * `param_name` - The name of the parameter to retrieve
    ///
    /// ## Returns
    ///
    /// Returns `Ok(&str)` with the parameter value if found, or
    /// `Err(HttpRequestError::MissingParam)` if not found.
    ///
    /// ## Example
    /// ```
    /// let req = ripress::context::HttpRequest::new();
    /// let id = req.get_params("id");
    /// println!("Id: {:?}", id);
    /// ```

    pub fn get_params(&self, param_name: &str) -> Result<&str, HttpRequestError> {
        let param = self.params.get(param_name).map(|v| v);

        match param {
            Some(param_str) => Ok(param_str),
            None => Err(HttpRequestError::MissingParam(param_name.to_string())),
        }
    }

    /// Adds data from the middleware into the request.
    ///
    /// ## Arguments
    ///
    /// * `key` - The key of the data to retrieve
    /// * `value` - The value of the data to retrieve
    ///
    /// ## Example
    /// ```
    /// let mut req = ripress::context::HttpRequest::new();
    /// req.set_data("id", "123");
    /// let id = req.get_data("id");
    /// println!("Id: {:?}", id);
    /// ```

    pub fn set_data(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    /// Returns data stored in the request by the middleware.
    ///
    /// ## Arguments
    ///
    /// * `key` - The key of the data to retrieve
    ///
    /// ## Returns
    ///
    /// Returns `Option<&String>` with the data value if found, or `None` if not found.
    ///
    /// ## Example
    /// ```
    /// let req = ripress::context::HttpRequest::new();
    /// let id = req.get_data("id");
    /// println!("Id: {:?}", id);
    /// ```

    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    pub fn get_all_data(&self) -> Option<&HashMap<String, String>> {
        Some(&self.data)
    }

    /// Returns header based on the key.
    ///
    /// ## Arguments
    ///
    /// * `header_name` - The name of the header to retrieve
    ///
    /// ## Returns
    ///
    /// Returns `Ok(&str)` with the header value if found, or
    /// `Err(HttpRequestError::MissingHeader)` if not found.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// let req = ripress::context::HttpRequest::new();
    /// let header = req.get_header("id");
    /// println!("header: {:?}", header.unwrap());
    /// ```

    pub fn get_header(&self, header_name: &str) -> Result<&str, HttpRequestError> {
        let header_name = header_name.to_lowercase();
        let header = self.headers.get(&header_name);

        match header {
            Some(header_str) => Ok(header_str),
            None => Err(HttpRequestError::MissingHeader(header_name)),
        }
    }

    /// Returns query parameters.
    ///
    /// ## Arguments
    ///     
    /// * `query_name` - The name of the query parameter to retrieve
    ///
    /// ## Returns
    ///
    /// Returns `Ok(&str)` with the query parameter value if found, or
    /// `Err(HttpRequestError::MissingParam)` if not found.
    ///
    /// ## Example
    /// ```
    /// let req = ripress::context::HttpRequest::new();
    /// let id = req.get_query("id");
    /// println!("Id: {:?}", id);
    /// ```

    pub fn get_query(&self, query_name: &str) -> Result<&str, HttpRequestError> {
        let query = self.queries.get(query_name).map(|v| v);

        match query {
            Some(query_str) => Ok(query_str),
            None => Err(HttpRequestError::MissingQuery(query_name.to_string())),
        }
    }

    /// Returns the protocol on which the request was made (http or https)
    ///
    /// ## Returns
    ///
    /// Returns &str with the protocol value
    ///
    /// ## Example
    /// ```
    /// let req = ripress::context::HttpRequest::new();
    /// let body = req.get_protocol();
    /// ```

    pub fn get_protocol(&self) -> &str {
        &self.protocol
    }

    /// Returns a bool indicating if request was made over https
    ///
    /// ## Returns
    ///
    /// Returns bool, that is true if protocol was https else false
    ///
    /// ## Example
    /// ```
    /// let req = ripress::context::HttpRequest::new();
    /// let body = req.is_secure();
    /// ```
    ///
    /// This function returns a boolean.
    /// Returns true if request was made over https

    pub fn is_secure(&self) -> bool {
        self.get_protocol() == "https"
    }

    /// Deserializes the request body as JSON into the specified type.
    ///
    /// ## Type Parameters
    ///
    /// * `J` - The type to deserialize into, must implement `DeserializeOwned`
    ///
    /// ## Returns
    ///
    /// Returns `Ok(J)` with the deserialized value if successful, or
    /// `Err(String)` with an error message if deserialization fails.
    ///
    /// ## Example
    /// ```rust
    /// use ripress::context::HttpRequest;
    /// use serde::{Deserialize, Serialize};
    ///
    /// ##[derive(Deserialize, Serialize)]
    /// struct LoginData {
    ///     username: String,
    ///     password: String
    /// }
    ///
    /// let req = HttpRequest::new();
    /// match req.json::<LoginData>() {
    ///     Ok(data) => println!("Login attempt for: {}", data.username),
    ///     Err(e) => println!("Invalid login data: {}", e)
    /// }
    /// ```

    pub fn json<J>(&self) -> Result<J, String>
    where
        J: serde::de::DeserializeOwned + serde::Serialize,
    {
        let body = &self.body;

        if body.content_type == RequestBodyType::JSON {
            if let RequestBodyContent::JSON(ref json_value) = body.content {
                match serde_json::from_value::<J>(json_value.clone()) {
                    Ok(serialized) => Ok(serialized),
                    Err(e) => Err(format!("Failed to deserialize JSON: {}", e)),
                }
            } else {
                Err(String::from("Invalid JSON content"))
            }
        } else {
            Err(String::from("Wrong body type"))
        }
    }

    /// Returns request's text body.
    ///
    /// ## Example
    /// ```
    /// let req = ripress::context::HttpRequest::new();
    /// let text = req.text().unwrap();
    /// println!("text : {:?}", text);
    /// ```
    ///
    /// This function returns the text body of the request.
    /// Returns an `Result<String>`, where `Ok(String)` contains the body if it is valid text, or `Err(error)` if it is not.

    pub fn text(&self) -> Result<String, String> {
        let body = &self.body;

        if body.content_type == RequestBodyType::TEXT {
            if let RequestBodyContent::TEXT(ref text_value) = body.content {
                Ok(text_value.clone())
            } else {
                Err(String::from("Invalid text content"))
            }
        } else {
            Err(String::from("Wrong body type"))
        }
    }

    /// Returns request's form_data body.
    ///
    /// ## Example
    /// ```no_run
    /// let req = ripress::context::HttpRequest::new();
    /// // Let' say form data was sent as key=value and key2=value2
    /// let form_data = req.form_data().unwrap();
    /// println!("key = : {:?}", form_data.get("key"));
    /// println!("key2 = : {:?}", form_data.get("key2"));
    /// ```
    ///
    /// This function returns a HashMap of the form data.
    /// Returns an `Result<HashMap<String, String>>`, where `Ok(HashMap<String, String>)` contains the form_data if it is valid form data, or `Err(error)` if it is not.

    pub fn form_data(&self) -> Result<HashMap<String, String>, String> {
        let mut form_data: HashMap<String, String> = HashMap::new();
        let body = &self.body;

        if body.content_type == RequestBodyType::FORM {
            if let RequestBodyContent::FORM(ref text_value) = body.content {
                serde_urlencoded::from_str::<HashMap<String, String>>(text_value)
                    .map_err(|e| e.to_string())?
                    .into_iter()
                    .for_each(|(k, v)| {
                        form_data.insert(k, v);
                    });
                Ok(form_data)
            } else {
                Err(String::from("Invalid form content"))
            }
        } else {
            Err(String::from("Wrong body type"))
        }
    }

    pub async fn from_hyper_request(
        req: &mut Request<hyper::body::Body>,
    ) -> Result<Self, hyper::Error> {
        let query_string = req.uri().query().unwrap_or("");

        let queries = url::form_urlencoded::parse(query_string.as_bytes())
            .filter_map(|(key, value)| Some((key.to_string(), value.to_string())))
            .collect::<HashMap<String, String>>();

        let ip = get_real_ip_hyper(&req);
        let origin_url = req.uri().to_string();
        let path = req.uri().path().to_string();

        let mut cookies_map = HashMap::new();
        let cookies = get_cookies(&req);

        cookies.iter().for_each(|cookie| {
            let (name, value) = (cookie.name(), cookie.value());
            cookies_map.insert(name.to_string(), value.to_string());
        });

        let mut headers: HashMap<String, String> = HashMap::new();

        req.headers().iter().for_each(|(key, value)| {
            headers.insert(key.to_string(), value.to_str().unwrap().to_string());
        });

        // if let Some(captures) = path.strip_prefix(req.uri().path()) {
        //     println!("User ID: {}", captures); // "123"
        // }

        // let params: HashMap<String, String> = req
        //     .match_info()
        //     .iter()
        //     .map(|(k, v)| (k.to_string(), v.to_string()))
        //     .collect();

        let content_type = req
            .headers()
            .get("Content-Type")
            .and_then(|val| val.to_str().ok())
            .map(determine_content_type)
            .unwrap_or(RequestBodyType::EMPTY);

        let protocol = req
            .headers()
            .get("x-forwarded-proto")
            .and_then(|val| val.to_str().ok())
            .unwrap_or("http")
            .to_string();

        let request_body = match content_type {
            RequestBodyType::FORM => {
                let body_bytes = to_bytes(req.body_mut()).await;

                let body_string = match body_bytes {
                    Ok(bytes) => std::str::from_utf8(&bytes).unwrap().to_string(),
                    Err(err) => return Err(err),
                };

                RequestBody {
                    content: RequestBodyContent::FORM(body_string.to_string()),
                    content_type: RequestBodyType::FORM,
                }
            }
            RequestBodyType::JSON => {
                let body_bytes = to_bytes(req.body_mut()).await;
                let body_json = match body_bytes {
                    Ok(bytes) => {
                        let s = std::str::from_utf8(&bytes).unwrap();
                        serde_json::from_str(s).unwrap()
                    }
                    Err(err) => return Err(err),
                };

                RequestBody {
                    content: RequestBodyContent::JSON(body_json),
                    content_type: RequestBodyType::JSON,
                }
            }
            RequestBodyType::TEXT => {
                let body_bytes = to_bytes(req.body_mut()).await;

                let body_string = match body_bytes {
                    Ok(bytes) => std::str::from_utf8(&bytes).unwrap().to_string(),
                    Err(err) => return Err(err),
                };

                RequestBody {
                    content: RequestBodyContent::FORM(body_string.to_string()),
                    content_type: RequestBodyType::FORM,
                }
            }
            RequestBodyType::EMPTY => RequestBody {
                content: RequestBodyContent::EMPTY,
                content_type: RequestBodyType::EMPTY,
            },
        };

        Ok(HttpRequest {
            params: HashMap::new(),
            queries,
            body: request_body,
            ip,
            method: HttpMethods::GET,
            origin_url,
            path,
            headers,
            cookies: cookies_map,
            protocol,
            data: HashMap::new(),
        })
    }
}

/// Determines the content type from a content-type header string.
///
/// ## Arguments
///
/// * `content_type` - The content-type header value
///
/// ## Returns
///
/// Returns the appropriate `RequestBodyType` enum variant.

pub(crate) fn determine_content_type(content_type: &str) -> RequestBodyType {
    if content_type == "application/json" {
        return RequestBodyType::JSON;
    } else if content_type == "application/x-www-form-urlencoded" {
        return RequestBodyType::FORM;
    } else {
        RequestBodyType::TEXT
    }
}

pub(crate) fn get_real_ip_hyper(req: &Request<hyper::Body>) -> String {
    req.headers()
        .get("X-Forwarded-For")
        .and_then(|val| val.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .unwrap_or(String::new())
}

pub(crate) fn get_cookies(req: &Request<Body>) -> Vec<Cookie<'_>> {
    let mut cookies = Vec::new();

    if let Some(header_value) = req.headers().get("cookie") {
        if let Ok(header_str) = header_value.to_str() {
            for cookie_str in header_str.split(';') {
                if let Ok(cookie) = Cookie::parse(cookie_str.trim()) {
                    cookies.push(cookie);
                }
            }
        }
    }

    cookies
}

#[cfg(test)]
impl HttpRequest {
    pub(crate) fn set_query(&mut self, key: &str, value: &str) {
        self.queries.insert(key.to_string(), value.to_string());
    }

    pub(crate) fn set_header(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }

    pub(crate) fn set_cookie(&mut self, key: &str, value: &str) {
        self.cookies.insert(key.to_string(), value.to_string());
    }

    pub(crate) fn set_param(&mut self, key: &str, value: &str) {
        self.params.insert(key.to_string(), value.to_string());
    }

    pub(crate) fn set_json<J>(&mut self, json: J, content_type: RequestBodyType)
    where
        J: serde::de::DeserializeOwned + serde::Serialize,
    {
        self.body.content_type = content_type;
        self.body.content = RequestBodyContent::JSON(serde_json::to_value(json).unwrap());
    }

    pub(crate) fn set_text(&mut self, text: &str, content_type: RequestBodyType) {
        self.body.content_type = content_type;
        self.body.content = RequestBodyContent::TEXT(text.to_string());
    }

    pub(crate) fn set_form(&mut self, key: &str, value: &str, content_type: RequestBodyType) {
        self.body.content_type = content_type;

        match &mut self.body.content {
            RequestBodyContent::FORM(existing) => {
                existing.push('&');
                existing.push_str(&format!("{key}={value}"));
            }
            _ => {
                self.body.content = RequestBodyContent::FORM(format!("{key}={value}"));
            }
        }
    }

    pub(crate) fn set_content_type(&mut self, content_type: RequestBodyType) {
        self.body.content_type = content_type;
    }

    pub(crate) fn set_method(&mut self, method: HttpMethods) {
        self.method = method;
    }

    pub(crate) fn set_ip(&mut self, ip: String) {
        self.ip = ip;
    }

    pub(crate) fn set_path(&mut self, path: String) {
        self.path = path;
    }

    pub(crate) fn set_origin_url(&mut self, origin_url: String) {
        self.origin_url = origin_url;
    }
}
