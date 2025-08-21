#![warn(missing_docs)]
use std::fmt::Display;

/// Represents the status code of an HTTP response.
///
/// This enum provides a type-safe representation of HTTP status codes with commonly
/// used variants and support for custom status codes. It implements bidirectional
/// conversion between numeric codes and enum variants.
///
/// # Examples
///
/// ```
/// use ripress::res::response_status::StatusCode;
///
/// // Create status codes
/// let success = StatusCode::Ok;
/// let not_found = StatusCode::NotFound;
/// let custom = StatusCode::Custom(418);
///
/// // Display status codes
/// println!("{}", success);    // "200 OK"
/// println!("{}", not_found);  // "404 Not Found"
/// println!("{}", custom);     // "418 Custom"
///
/// // Convert from numeric codes
/// let code = StatusCode::from_u16(404);
/// assert_eq!(code, StatusCode::NotFound);
///
/// // Convert to numeric codes
/// assert_eq!(StatusCode::Ok.as_u16(), 200);
/// ```
///
/// # Status Code Categories
///
/// ## 2xx Success
/// - [`Ok`](StatusCode::Ok) (200) - Request succeeded
/// - [`Created`](StatusCode::Created) (201) - Resource created successfully
/// - [`Accepted`](StatusCode::Accepted) (202) - Request accepted for processing
/// - [`NoContent`](StatusCode::NoContent) (204) - Request succeeded with no content
///
/// ## 3xx Redirection
/// - [`PermanentRedirect`](StatusCode::PermanentRedirect) (301) - Resource permanently moved
/// - [`Redirect`](StatusCode::Redirect) (302) - Resource temporarily moved
///
/// ## 4xx Client Error
/// - [`BadRequest`](StatusCode::BadRequest) (400) - Invalid request syntax
/// - [`Unauthorized`](StatusCode::Unauthorized) (401) - Authentication required
/// - [`Forbidden`](StatusCode::Forbidden) (403) - Access denied
/// - [`NotFound`](StatusCode::NotFound) (404) - Resource not found
/// - [`MethodNotAllowed`](StatusCode::MethodNotAllowed) (405) - HTTP method not supported
/// - [`Conflict`](StatusCode::Conflict) (409) - Request conflicts with current state
/// - [`PayloadTooLarge`](StatusCode::PayloadTooLarge) (413) - Request payload too large
/// - [`TooManyRequests`](StatusCode::TooManyRequests) (429) - Too many requests
///
/// ## 5xx Server Error
/// - [`InternalServerError`](StatusCode::InternalServerError) (500) - Generic server error
/// - [`NotImplemented`](StatusCode::NotImplemented) (501) - Server doesn't support functionality
/// - [`BadGateway`](StatusCode::BadGateway) (502) - Invalid response from upstream server
/// - [`ServiceUnavailable`](StatusCode::ServiceUnavailable) (503) - Server temporarily unavailable
///
/// ## Custom Status Codes
/// - [`Custom`](StatusCode::Custom) - For non-standard or application-specific codes
///
/// # Design Notes
///
/// This enum is designed as a zero-cost abstraction that compiles down to efficient
/// representations. All variants are Copy and require no heap allocations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCode {
    /// 200 OK
    ///
    /// The request succeeded. The meaning of the success depends on the HTTP method:
    /// - GET: The resource has been fetched and transmitted in the message body
    /// - HEAD: The representation headers are included without any message body
    /// - PUT or POST: The resource describing the result of the action is transmitted
    /// - TRACE: The message body contains the request message as received by the server
    Ok,

    /// 201 Created
    ///
    /// The request succeeded, and a new resource was created as a result.
    /// This is typically the response sent after POST requests, or some PUT requests.
    Created,

    /// 202 Accepted
    ///
    /// The request has been received but not yet acted upon. It is noncommittal,
    /// since there is no way in HTTP to later send an asynchronous response
    /// indicating the outcome of the request.
    Accepted,

    /// 204 No Content
    ///
    /// There is no content to send for this request, but the headers may be useful.
    /// The user agent may update its cached headers for this resource with the new ones.
    NoContent,

    /// 301 Moved Permanently
    ///
    /// The URL of the requested resource has been changed permanently.
    /// The new URL is given in the response.
    PermanentRedirect,

    /// 302 Found (Temporary Redirect)
    ///
    /// This response code means that the URI of requested resource has been changed
    /// temporarily. Further changes in the URI might be made in the future.
    Redirect,

    /// 400 Bad Request
    ///
    /// The server cannot or will not process the request due to an apparent client
    /// error (e.g., malformed request syntax, size too large, invalid request message framing).
    BadRequest,

    /// 401 Unauthorized
    ///
    /// Although the HTTP standard specifies "unauthorized", semantically this response
    /// means "unauthenticated". The client must authenticate itself to get the requested response.
    Unauthorized,

    /// 403 Forbidden
    ///
    /// The client does not have access rights to the content; that is, it is unauthorized,
    /// so the server is refusing to give the requested resource.
    Forbidden,

    /// 404 Not Found
    ///
    /// The server can not find the requested resource. In the browser, this means the
    /// URL is not recognized. This is probably the most famous status code.
    NotFound,

    /// 405 Method Not Allowed
    ///
    /// The request method is known by the server but is not supported by the target resource.
    /// For example, an API may not allow calling DELETE to remove a resource.
    MethodNotAllowed,

    /// 409 Conflict
    ///
    /// This response is sent when a request conflicts with the current state of the server.
    /// Often used in REST APIs when trying to create a resource that already exists.
    Conflict,

    /// 413 Payload Too Large
    ///
    /// This response is sent when the request payload is too large for the server to handle.
    /// The server may choose to respond with a 413 status code and include a Retry-After header
    /// to indicate how long the user should wait before making a new request.
    PayloadTooLarge,

    /// 429 Too Many Requests
    ///
    /// This response is sent when a request is rejected due to the user exceeding the rate limit.
    /// The server may choose to respond with a 429 status code and include a Retry-After header
    /// to indicate how long the user should wait before making a new request.
    TooManyRequests,

    /// 500 Internal Server Error
    ///
    /// The server has encountered a situation it does not know how to handle.
    /// This is a generic error message when no more specific message is suitable.
    InternalServerError,

    /// 501 Not Implemented
    ///
    /// The request method is not supported by the server and cannot be handled.
    /// The only methods that servers are required to support are GET and HEAD.
    NotImplemented,

    /// 502 Bad Gateway
    ///
    /// This error response means that the server, while working as a gateway to get
    /// a response needed to handle the request, got an invalid response.
    BadGateway,

    /// 503 Service Unavailable
    ///
    /// The server is not ready to handle the request. Common causes are a server
    /// that is down for maintenance or that is overloaded.
    ServiceUnavailable,

    /// A custom status code with a given u16 value
    ///
    /// This variant allows representing any HTTP status code, including:
    /// - Non-standard codes (e.g., 418 I'm a teapot)
    /// - Application-specific codes
    /// - Future HTTP status codes not yet included as named variants
    ///
    /// # Examples
    ///
    /// ```
    /// use ripress::res::response_status::StatusCode;
    ///
    /// let teapot = StatusCode::Custom(418);
    /// assert_eq!(teapot.as_u16(), 418);
    /// println!("{}", teapot); // "418 Custom"
    /// ```
    Custom(u16),
}

impl Display for StatusCode {
    /// Formats the status code as "code description" (e.g., "404 Not Found").
    ///
    /// This follows the standard HTTP status line format and is suitable for
    /// logging, debugging, or displaying to users.
    ///
    /// # Examples
    ///
    /// ```
    /// use ripress::res::response_status::StatusCode;
    ///
    /// assert_eq!(format!("{}", StatusCode::Ok), "200 OK");
    /// assert_eq!(format!("{}", StatusCode::NotFound), "404 Not Found");
    /// assert_eq!(format!("{}", StatusCode::Custom(418)), "418 Custom");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code = self.as_u16();
        write!(
            f,
            "{} {}",
            code,
            match self {
                StatusCode::Ok => "OK",
                StatusCode::Created => "Created",
                StatusCode::Accepted => "Accepted",
                StatusCode::NoContent => "No Content",
                StatusCode::Redirect => "Found",
                StatusCode::PermanentRedirect => "Moved Permanently",
                StatusCode::BadRequest => "Bad Request",
                StatusCode::Unauthorized => "Unauthorized",
                StatusCode::Forbidden => "Forbidden",
                StatusCode::NotFound => "Not Found",
                StatusCode::MethodNotAllowed => "Method Not Allowed",
                StatusCode::Conflict => "Conflict",
                StatusCode::PayloadTooLarge => "Payload Too Large",
                StatusCode::TooManyRequests => "Too Many Requests",
                StatusCode::InternalServerError => "Internal Server Error",
                StatusCode::NotImplemented => "Not Implemented",
                StatusCode::BadGateway => "Bad Gateway",
                StatusCode::ServiceUnavailable => "Service Unavailable",
                StatusCode::Custom(_code) => "Custom",
            }
        )
    }
}

impl StatusCode {
    /// Returns the numeric HTTP status code as a u16.
    ///
    /// This method provides the underlying numeric value for any status code,
    /// whether it's a named variant or a custom code.
    ///
    /// # Examples
    ///
    /// ```
    /// use ripress::res::response_status::StatusCode;
    ///
    /// assert_eq!(StatusCode::Ok.as_u16(), 200);
    /// assert_eq!(StatusCode::NotFound.as_u16(), 404);
    /// assert_eq!(StatusCode::Custom(418).as_u16(), 418);
    /// ```
    pub fn as_u16(&self) -> u16 {
        match self {
            StatusCode::Ok => 200,
            StatusCode::Created => 201,
            StatusCode::Accepted => 202,
            StatusCode::NoContent => 204,

            StatusCode::Redirect => 302,
            StatusCode::PermanentRedirect => 301,

            StatusCode::BadRequest => 400,
            StatusCode::Unauthorized => 401,
            StatusCode::Forbidden => 403,
            StatusCode::NotFound => 404,
            StatusCode::MethodNotAllowed => 405,
            StatusCode::Conflict => 409,
            StatusCode::PayloadTooLarge => 413,
            StatusCode::TooManyRequests => 429,

            StatusCode::InternalServerError => 500,
            StatusCode::NotImplemented => 501,
            StatusCode::BadGateway => 502,
            StatusCode::ServiceUnavailable => 503,

            StatusCode::Custom(code) => *code,
        }
    }

    /// Creates a StatusCode from a numeric HTTP status code.
    ///
    /// This method returns a named variant if the code matches a known HTTP status code,
    /// otherwise it returns `StatusCode::Custom(code)`. This ensures that any valid
    /// u16 value can be represented as a StatusCode.
    ///
    /// # Examples
    ///
    /// ```
    /// use ripress::res::response_status::StatusCode;
    ///
    /// // Known status codes return named variants
    /// assert_eq!(StatusCode::from_u16(200), StatusCode::Ok);
    /// assert_eq!(StatusCode::from_u16(404), StatusCode::NotFound);
    ///
    /// // Unknown status codes return Custom variants
    /// assert_eq!(StatusCode::from_u16(418), StatusCode::Custom(418));
    /// assert_eq!(StatusCode::from_u16(999), StatusCode::Custom(999));
    /// ```
    ///
    /// # Performance
    ///
    /// This method compiles to an efficient jump table for known status codes
    /// and has O(1) time complexity.
    pub fn from_u16(code: u16) -> StatusCode {
        match code {
            200 => StatusCode::Ok,
            201 => StatusCode::Created,
            202 => StatusCode::Accepted,
            204 => StatusCode::NoContent,

            302 => StatusCode::Redirect,
            301 => StatusCode::PermanentRedirect,

            400 => StatusCode::BadRequest,
            401 => StatusCode::Unauthorized,
            403 => StatusCode::Forbidden,
            404 => StatusCode::NotFound,
            405 => StatusCode::MethodNotAllowed,
            409 => StatusCode::Conflict,
            413 => StatusCode::PayloadTooLarge,
            429 => StatusCode::TooManyRequests,

            500 => StatusCode::InternalServerError,
            501 => StatusCode::NotImplemented,
            502 => StatusCode::BadGateway,
            503 => StatusCode::ServiceUnavailable,

            other => StatusCode::Custom(other),
        }
    }

    /// Returns `true` if the status code indicates success (2xx range).
    ///
    /// # Examples
    ///
    /// ```
    /// use ripress::res::response_status::StatusCode;
    ///
    /// assert!(StatusCode::Ok.is_success());
    /// assert!(StatusCode::Created.is_success());
    /// assert!(!StatusCode::NotFound.is_success());
    /// assert!(StatusCode::Custom(299).is_success());
    /// ```
    pub fn is_success(&self) -> bool {
        matches!(self.as_u16(), 200..=299)
    }

    /// Returns `true` if the status code indicates a redirection (3xx range).
    ///
    /// # Examples
    ///
    /// ```
    /// use ripress::res::response_status::StatusCode;
    ///
    /// assert!(StatusCode::Redirect.is_redirection());
    /// assert!(StatusCode::PermanentRedirect.is_redirection());
    /// assert!(!StatusCode::Ok.is_redirection());
    /// assert!(StatusCode::Custom(399).is_redirection());
    /// ```
    pub fn is_redirection(&self) -> bool {
        matches!(self.as_u16(), 300..=399)
    }

    /// Returns `true` if the status code indicates a client error (4xx range).
    ///
    /// # Examples
    ///
    /// ```
    /// use ripress::res::response_status::StatusCode;
    ///
    /// assert!(StatusCode::BadRequest.is_client_error());
    /// assert!(StatusCode::NotFound.is_client_error());
    /// assert!(!StatusCode::Ok.is_client_error());
    /// assert!(StatusCode::Custom(418).is_client_error());
    /// ```
    pub fn is_client_error(&self) -> bool {
        matches!(self.as_u16(), 400..=499)
    }

    /// Returns `true` if the status code indicates a server error (5xx range).
    ///
    /// # Examples
    ///
    /// ```
    /// use ripress::res::response_status::StatusCode;
    ///
    /// assert!(StatusCode::InternalServerError.is_server_error());
    /// assert!(StatusCode::BadGateway.is_server_error());
    /// assert!(!StatusCode::Ok.is_server_error());
    /// assert!(StatusCode::Custom(599).is_server_error());
    /// ```
    pub fn is_server_error(&self) -> bool {
        matches!(self.as_u16(), 500..=599)
    }

    /// Returns `true` if the status code indicates an informational response (1xx range).
    ///
    /// Note: This crate doesn't include named variants for 1xx codes, but this method
    /// will return `true` for custom codes in the 1xx range.
    ///
    /// # Examples
    ///
    /// ```
    /// use ripress::res::response_status::StatusCode;
    ///
    /// assert!(!StatusCode::Ok.is_informational());
    /// assert!(StatusCode::Custom(100).is_informational()); // Continue
    /// assert!(StatusCode::Custom(101).is_informational()); // Switching Protocols
    /// ```
    pub fn is_informational(&self) -> bool {
        matches!(self.as_u16(), 100..=199)
    }

    /// Returns the canonical reason phrase for this status code.
    ///
    /// This returns the standard HTTP reason phrase associated with the status code,
    /// which is the same text used in the Display implementation but without the numeric code.
    ///
    /// # Examples
    ///
    /// ```
    /// use ripress::res::response_status::StatusCode;
    ///
    /// assert_eq!(StatusCode::Ok.canonical_reason(), "OK");
    /// assert_eq!(StatusCode::NotFound.canonical_reason(), "Not Found");
    /// assert_eq!(StatusCode::Custom(418).canonical_reason(), "Custom");
    /// ```
    pub fn canonical_reason(&self) -> &'static str {
        match self {
            StatusCode::Ok => "OK",
            StatusCode::Created => "Created",
            StatusCode::Accepted => "Accepted",
            StatusCode::NoContent => "No Content",
            StatusCode::Redirect => "Found",
            StatusCode::PermanentRedirect => "Moved Permanently",
            StatusCode::BadRequest => "Bad Request",
            StatusCode::Unauthorized => "Unauthorized",
            StatusCode::Forbidden => "Forbidden",
            StatusCode::NotFound => "Not Found",
            StatusCode::MethodNotAllowed => "Method Not Allowed",
            StatusCode::Conflict => "Conflict",
            StatusCode::PayloadTooLarge => "Payload Too Large",
            StatusCode::TooManyRequests => "Too Many Requests",
            StatusCode::InternalServerError => "Internal Server Error",
            StatusCode::NotImplemented => "Not Implemented",
            StatusCode::BadGateway => "Bad Gateway",
            StatusCode::ServiceUnavailable => "Service Unavailable",
            StatusCode::Custom(_) => "Custom",
        }
    }
}
