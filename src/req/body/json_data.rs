//! JSON body extraction and deserialization.
//!
//! This module provides the [`JsonBody`] wrapper and [`FromJson`] trait
//! for type-safe JSON extraction from HTTP requests.

use crate::{helpers::FromRequest, req::body::RequestBodyContent};
use std::ops::Deref;

/// A wrapper around a deserialized JSON body.
///
/// Use this in handler signatures to automatically extract and deserialize
/// JSON request bodies.
///
/// # Example
/// ```ignore
/// fn handler(body: JsonBody<MyStruct>) {
///     // Access the inner value
///     let data: &MyStruct = &*body;
/// }
/// ```

pub struct JsonBody<T>(T);

impl<T: FromJson> FromRequest for JsonBody<T> {
    type Error = String;

    fn from_request(req: &crate::req::HttpRequest) -> Result<Self, Self::Error> {
        let body = &req.body.content;
        Ok(Self(T::from_json(body)?))
    }
}

impl<T> Deref for JsonBody<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Trait for types that can be deserialized from JSON request bodies.
///
/// This trait is typically derived using `#[derive(FromJson)]` from the
/// `ripress-derive` crate.
pub trait FromJson: Sized {
    /// Attempt to deserialize an instance of this type from a JSON request body.
    ///
    /// # Arguments
    ///
    /// * `data` - The parsed body content, expected to be in JSON form.
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` if deserialization succeeds.
    /// * `Err(String)` if deserialization fails (e.g., invalid/missing JSON).
    fn from_json(data: &RequestBodyContent) -> Result<Self, String>;
}
