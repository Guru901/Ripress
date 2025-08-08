/// Represents the status code of an HTTP response.
///
/// This enum is used to represent the status code of an HTTP response. It
/// provides a set of common status codes and a way to create custom status
/// codes.
///
/// # Variants
///
/// - `Ok`: 200 OK
/// - `Created`: 201 Created
/// - `Accepted`: 202 Accepted
/// - `NoContent`: 204 No Content
/// - `Redirect`: 301 Moved Permanently
/// - `PermanentRedirect`: 301 Moved Permanently
/// - `BadRequest`: 400 Bad Request
/// - `Unauthorized`: 401 Unauthorized
/// - `Forbidden`: 403 Forbidden
/// - `NotFound`: 404 Not Found
/// - `MethodNotAllowed`: 405 Method Not Allowed
/// - `Conflict`: 409 Conflict
/// - `InternalServerError`: 500 Internal Server Error
/// - `NotImplemented`: 501 Not Implemented
/// - `BadGateway`: 502 Bad Gateway
/// - `ServiceUnavailable`: 503 Service Unavailable
/// - `Custom(u16)`: A custom status code with a given u16 value

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCode {
    /// 200 OK
    Ok,

    /// 201 Created
    Created,

    /// 202 Accepted
    Accepted,

    /// 204 No Content
    NoContent,

    /// 301 Permanent Redirect
    PermanentRedirect,

    /// 302 Redirect
    Redirect,

    /// 400 Bad Request
    BadRequest,

    /// 401 Unauthorized
    Unauthorized,

    /// 403 Forbidden
    Forbidden,

    /// 404 Not Found
    NotFound,

    /// 405 Method Not Allowed
    MethodNotAllowed,

    /// 409 Conflict
    Conflict,

    /// 500 Internal Server Error
    InternalServerError,

    /// 501 Not Implemented
    NotImplemented,

    /// 502 Bad Gateway
    BadGateway,

    /// 503 Service Unavailable
    ServiceUnavailable,

    /// A custom status code with a given u16 value
    Custom(u16),
}

impl StatusCode {
    /// Returns the u16 value of the status code.
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

            StatusCode::InternalServerError => 500,
            StatusCode::NotImplemented => 501,
            StatusCode::BadGateway => 502,
            StatusCode::ServiceUnavailable => 503,

            StatusCode::Custom(code) => *code,
        }
    }

    /// Creates the status code from a given u16 value.
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

            500 => StatusCode::InternalServerError,
            501 => StatusCode::NotImplemented,
            502 => StatusCode::BadGateway,
            503 => StatusCode::ServiceUnavailable,

            other => StatusCode::Custom(other),
        }
    }
}
