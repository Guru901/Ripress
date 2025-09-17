#![warn(missing_docs)]

//! # Ripress
//!
//! Ripress is a lightweight, modular web framework for building HTTP APIs and web applications in Rust.
//! It provides a simple and flexible API for defining routes, handling requests and responses, and composing middleware.
//!
//! ## Modules
//!
//! - [`app`] - The main application struct and its methods for configuring and running your server.
//! - [`req`] - The HTTP request struct and utilities for extracting data from requests.
//! - [`res`] - The HTTP response struct and methods for building responses.
//! - [`context`] - Convenient re-exports of common request and response types.
//! - [`helpers`] - Utility functions and helpers for common web tasks.
//! - [`middlewares`] - Built-in middleware modules for CORS, logging, file uploads, and rate limiting.
//! - [`router`] - The router struct and routing logic for organizing endpoints.
//! - [`types`] - Core types, traits, and enums used throughout the framework.

/// The main application struct and its methods for configuring and running your server.
///
/// See [`app::App`] for details.
pub mod app;

/// The HTTP request struct and its methods for extracting data from requests.
///
/// See [`req::HttpRequest`] for details.
pub mod req;

/// The HTTP response struct and its methods for building responses.
///
/// See [`res::HttpResponse`] for details.
pub mod res;

/// Common context types for handler functions.
///
/// Re-exports [`HttpRequest`] and [`HttpResponse`] for convenience.
pub mod context {
    pub use super::req::HttpRequest;
    pub use super::res::HttpResponse;
}

/// Utility functions and helpers for common web tasks.
pub mod helpers;

/// Built-in middleware modules for CORS, logging, file uploads, and rate limiting.
pub mod middlewares;

/// The router struct and routing logic for organizing endpoints.
pub mod router;

/// Core types, traits, and enums used throughout the framework.
pub mod types;

/// Internal test module for framework testing.
mod tests;

/// Error types and utilities for the Ripress framework.
///
/// This module provides structured error types, error categories, and conversion utilities
/// for handling errors throughout the framework. It includes the [`RipressError`] struct,
/// the [`RipressErrorKind`] enum for classifying errors, and conversions from lower-level
/// errors such as query parameter and route parameter parsing failures.
///
/// See [`error::RipressError`] and [`error::RipressErrorKind`] for details.
pub mod error;
