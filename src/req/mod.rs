use crate::types::{HttpMethods, RequestBody, RequestBodyContent, RequestBodyType};
use actix_web::HttpMessage;
use futures::StreamExt;
use std::collections::HashMap;

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
/// println!("Method: {:?}", req.method);
/// println!("Path: {}", req.path);
/// println!("Client IP: {:?}", req.ip);
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

pub struct HttpRequest {
    /// Dynamic route parameters extracted from the URL.
    pub params: HashMap<String, String>,

    /// Query parameters from the request URL.
    pub query_params: HashMap<String, String>,

    /// The full URL of the incoming request.
    pub origin_url: String,

    /// The HTTP method used for the request (e.g., GET, POST, PUT, DELETE).
    pub method: HttpMethods,

    /// The IP address of the client making the request.
    pub ip: String,

    /// The requested endpoint path.
    pub path: String,

    /// Protocol of the request (HTTP or HTTPs)
    pub protocol: String,

    /// The request's headers
    headers: HashMap<String, String>,

    /// The request's cookies
    pub cookies: HashMap<String, String>,

    // The Data set by middleware in the request to be used in the route handler
    data: HashMap<String, String>,

    /// The request body, which may contain JSON, text, or form data.
    body: RequestBody,
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
    /// assert_eq!(req.method, HttpMethods::GET);
    /// ```

    pub fn new() -> Self {
        HttpRequest {
            origin_url: String::new(),
            params: HashMap::new(),
            query_params: HashMap::new(),
            method: HttpMethods::GET,
            ip: String::new(),
            path: String::new(),
            protocol: String::new(),
            headers: HashMap::new(),
            data: HashMap::new(),
            body: RequestBody::new_text(""),
            cookies: HashMap::new(),
        }
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

    pub fn get_header<T: Into<String>>(&self, header_name: T) -> Option<&String> {
        self.headers.get(&header_name.into().to_lowercase())
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

    pub fn set_data<T: Into<String>>(&mut self, data_key: T, data_value: T) {
        self.data.insert(data_key.into(), data_value.into());
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

    pub fn get_data<T: Into<String>>(&self, data_key: T) -> Option<&String> {
        self.data.get(&data_key.into())
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

    pub async fn from_actix_request(
        req: actix_web::HttpRequest,
        mut payload: actix_web::web::Payload,
    ) -> Result<Self, actix_web::Error> {
        let origin_url = req.full_url().to_string();

        let params: HashMap<String, String> = req
            .match_info()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        let method = req.method().as_str();

        let method = match method {
            "GET" => HttpMethods::GET,
            "POST" => HttpMethods::POST,
            "HEAD" => HttpMethods::HEAD,
            "PUT" => HttpMethods::PUT,
            _ => HttpMethods::GET,
        };

        let path = req.path().to_string();

        let ip = req
            .headers()
            .get("X-Forwarded-For")
            .and_then(|val| val.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
            .unwrap_or_else(|| {
                req.peer_addr()
                    .map(|addr| addr.ip().to_string())
                    .unwrap_or("unknown".to_string())
            });
        let protocol = req.connection_info().scheme().to_string();

        let mut headers = HashMap::new();

        req.headers().iter().for_each(|f| {
            let header_name = f.0.to_string();
            let header_value = f.1.to_str().unwrap().to_string();
            headers.insert(header_name, header_value);
        });

        let mut cookies: HashMap<String, String> = HashMap::new();

        req.cookies().iter().for_each(|cookie| {
            if let Some(first_cookie) = cookie.get(0) {
                let (name, value) = (first_cookie.name(), first_cookie.value());
                cookies.insert(name.to_string(), value.to_string());
            }
        });

        let query_string = req.query_string();

        let query_params = url::form_urlencoded::parse(query_string.as_bytes())
            .filter_map(|(key, value)| Some((key.to_string(), value.to_string())))
            .collect::<HashMap<String, String>>();

        let mut body = actix_web::web::BytesMut::new();

        let content_type = match req.content_type() {
            "application/json" => RequestBodyType::JSON,
            "application/x-www-form-urlencoded" => RequestBodyType::FORM,
            _ => RequestBodyType::TEXT,
        };

        while let Some(chunk) = payload.next().await {
            let chunk = chunk?;
            if (body.len() + chunk.len()) > 262_144 {
                return Err(actix_web::error::ErrorBadRequest("Body too large"));
            }
            body.extend_from_slice(&chunk);
        }

        let request_body = match content_type {
            RequestBodyType::FORM => {
                let form_data = match std::str::from_utf8(&body) {
                    Ok(s) => s.to_string(),
                    Err(_) => {
                        return Err(actix_web::error::ErrorBadRequest("Invalid UTF-8 sequence"));
                    }
                };

                RequestBody::new_form(form_data)
            }
            RequestBodyType::JSON => {
                let body_json = match std::str::from_utf8(&body) {
                    Ok(s) => match serde_json::from_str::<serde_json::Value>(s) {
                        Ok(json) => json,
                        Err(e) => {
                            return Err(actix_web::error::ErrorBadRequest(format!(
                                "Invalid JSON: {}",
                                e
                            )));
                        }
                    },
                    Err(_) => {
                        return Err(actix_web::error::ErrorBadRequest("Invalid UTF-8 sequence"));
                    }
                };

                RequestBody::new_json(body_json)
            }
            RequestBodyType::TEXT => {
                let body_string = match std::str::from_utf8(&body) {
                    Ok(s) => s.to_string(),
                    Err(_) => {
                        return Err(actix_web::error::ErrorBadRequest("Invalid UTF-8 sequence"));
                    }
                };

                RequestBody::new_text(body_string)
            }
        };

        Ok(HttpRequest {
            params,
            query_params,
            origin_url,
            method,
            ip,
            path,
            protocol,
            headers,
            data: HashMap::new(),
            body: request_body,
            cookies,
        })
    }
}
