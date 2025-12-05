use std::fmt::Display;

use crate::req::{
    body::text_data::TextDataError, query_params::QueryParamError, route_params::ParamError,
};

#[derive(Debug, PartialEq, Eq)]
pub enum RipressErrorKind {
    IO,
    ParseError,
    InvalidInput,
    NotFound,
}

impl Display for RipressErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RipressErrorKind::IO => write!(f, "IO error"),
            RipressErrorKind::ParseError => write!(f, "Parse error"),
            RipressErrorKind::InvalidInput => write!(f, "Invalid input"),
            RipressErrorKind::NotFound => write!(f, "Not found"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RipressError {
    pub kind: RipressErrorKind,
    pub message: String,
}

impl RipressError {
    pub fn new(kind: RipressErrorKind, message: String) -> Self {
        Self { kind, message }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn kind(&self) -> &RipressErrorKind {
        &self.kind
    }
}

impl Display for RipressError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RipressError: {{ message: {}, kind: {} }}",
            self.message, self.kind
        )
    }
}

impl From<std::io::Error> for RipressError {
    fn from(err: std::io::Error) -> Self {
        Self {
            kind: RipressErrorKind::IO,
            message: err.to_string(),
        }
    }
}

impl From<std::string::FromUtf8Error> for RipressError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Self {
            kind: RipressErrorKind::ParseError,
            message: err.to_string(),
        }
    }
}

impl From<QueryParamError> for RipressError {
    fn from(value: QueryParamError) -> Self {
        match value {
            QueryParamError::NotFound(param) => Self {
                kind: RipressErrorKind::NotFound,
                message: format!("Query Param '{}' not found", param),
            },
            QueryParamError::ParseError {
                param,
                value,
                target_type,
            } => Self {
                kind: RipressErrorKind::ParseError,
                message: format!(
                    "Failed to parse '{}' from: {} to:'{}'",
                    param, target_type, value
                ),
            },
        }
    }
}

impl From<ParamError> for RipressError {
    fn from(value: ParamError) -> Self {
        match value {
            ParamError::NotFound(param) => Self {
                kind: RipressErrorKind::NotFound,
                message: format!("Route Param '{}' not found", param),
            },
            ParamError::ParseError {
                param,
                value,
                target_type,
            } => Self {
                kind: RipressErrorKind::ParseError,
                message: format!(
                    "Failed to parse route param '{}' from: {} to: '{}'",
                    param, target_type, value
                ),
            },
        }
    }
}

impl From<TextDataError> for RipressError {
    fn from(value: TextDataError) -> Self {
        match value {
            TextDataError::InvalidUtf8(utf8_error) => Self {
                kind: RipressErrorKind::ParseError,
                message: utf8_error.to_string(),
            },
            TextDataError::TooLarge { size, limit } => Self {
                kind: RipressErrorKind::InvalidInput,
                message: format!("Text too large: {} bytes (limit: {} bytes)", size, limit),
            },
        }
    }
}
