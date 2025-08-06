#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCode {
    Ok,        // 200
    Created,   // 201
    Accepted,  // 202
    NoContent, // 204

    Redirect,          // 301
    PermanentRedirect, // 302

    BadRequest,       // 400
    Unauthorized,     // 401
    Forbidden,        // 403
    NotFound,         // 404
    MethodNotAllowed, // 405
    Conflict,         // 409

    InternalServerError, // 500
    NotImplemented,      // 501
    BadGateway,          // 502
    ServiceUnavailable,  // 503

    Custom(u16), // Custom status
}

impl StatusCode {
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
