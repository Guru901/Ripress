//! Middleware next-handler for chaining middleware functions in the request/response pipeline.
//!
//! This module provides the [`Next`] struct which allows middleware functions to
//! pass control to the next middleware in the chain. It also manages task-local
//! storage for pending headers and cookies that are collected during middleware
//! execution and applied to the final response.

use crate::{
    req::HttpRequest,
    res::{response_cookie::Cookie, HttpResponse},
};
use std::cell::RefCell;

tokio::task_local! {
    pub(crate) static PENDING_HEADERS: RefCell<Vec<(String, String)>>;
    pub(crate) static PENDING_COOKIES: RefCell<Vec<Cookie>>;
}

/// A marker type for calling the next middleware in the chain.
///
/// `Next` is passed to middleware functions to allow them to invoke the
/// next middleware and/or the final route handler. When invoked, it collects
/// any headers and cookies from the response and stores them in task-local
/// storage for later application to the final HTTP response.
///
/// # Construction
///
/// `Next` can be constructed via `Next::default()` or `Default::default()`:
///
/// ```ignore
/// let next = Next::default();
/// let next = Default::default();
/// ```
///
/// # Example
///
/// ```ignore
/// async fn logging_middleware(
///     req: HttpRequest,
///     res: HttpResponse,
///     next: Next,
/// ) -> (HttpRequest, Option<HttpResponse>) {
///     println!("Request: {}", req.path());
///     let (req, res) = next.call(req, res).await;
///     println!("Response status: {:?}", res.as_ref().map(|r| r.status()));
///     (req, res)
/// }
/// ```
#[derive(Clone, Default)]
pub struct Next;

impl Next {
    /// Calls the next middleware in the chain.
    ///
    /// This method invokes the next middleware or the final route handler.
    /// Any headers or cookies set in the response are collected and stored
    /// in task-local storage (`PENDING_HEADERS` and `PENDING_COOKIES`) for
    /// later application to the final HTTP response.
    ///
    /// # Arguments
    ///
    /// * `req` - The HTTP request to pass to the next middleware
    /// * `res` - The HTTP response (typically containing initial headers/cookies)
    ///
    /// # Returns
    ///
    /// Returns a tuple of `(HttpRequest, Option<HttpResponse>)` where:
    /// - The first element is the (possibly modified) request
    /// - The second element is `None` if the middleware chain should continue,
    ///   or `Some(HttpResponse)` if a middleware returned early
    ///
    /// # Panics
    ///
    /// Panics if the task-local storage for headers or cookies cannot be accessed,
    /// which typically indicates the code is being called outside a proper async task context.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let (req, res) = next.call(req, res).await;
    /// if let Some(response) = res {
    ///     return (req, Some(response));
    /// }
    /// // Otherwise, continue to the next middleware
    /// ```
    pub async fn call(
        &self,
        req: HttpRequest,
        res: HttpResponse,
    ) -> (HttpRequest, Option<HttpResponse>) {
        let cookies = res.cookies;
        let headers = res.headers;

        // Store all pending headers from the response in task-local storage
        // for later application to the final response
        PENDING_HEADERS.try_with(|pending| {
            let mut pending = pending.borrow_mut();

            for (k, v) in headers.iter() {
                pending.push((k.to_string(), v.to_string()));
            }
        }).expect("Failed to access task-local storage for pending headers");

        // Store all pending cookies from the response in task-local storage
        // for later application to the final response
        PENDING_COOKIES.try_with(move |pending| {
            let mut pending = pending.borrow_mut();

            for cookie in cookies {
                pending.push(cookie);
            }
        }).expect("Failed to access task-local storage for pending cookies");

        // Return the request unchanged and None to allow the next middleware to handle it
        (req, None)
    }
}
