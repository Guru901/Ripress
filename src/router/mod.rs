#![warn(missing_docs)]
use crate::{
    app::App,
    types::{RouterFns, Routes},
};
use std::collections::HashMap;

/// A modular router for grouping and mounting routes under a common base path.
///
/// The `Router` struct allows you to organize related routes together and mount them
/// onto an application at a specified base path. This is useful for building APIs
/// with versioning, grouping endpoints, or composing applications from multiple routers.
///
/// # Example
///
/// ```
/// use ripress::{router::Router, context::{HttpRequest, HttpResponse}, app::App};
/// use ripress::types::RouterFns;
///
/// async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
///     res.ok().text("Hello, World!")
/// }
///
/// let mut router = Router::new("/api");
/// router.get("/hello", handler);
/// let mut app = App::new();
/// router.register(&mut app);
/// ```
pub struct Router {
    /// The base path on which the router will be mounted to the app.
    ///
    /// All routes registered with this router will be prefixed with this path
    /// when mounted to an application.
    base_path: &'static str,

    /// The collection of routes registered on this router.
    ///
    /// This is a map from route paths (relative to the base path) to their
    /// associated HTTP method handlers.
    routes: Routes,
}

impl Router {
    /// Creates a new `Router` instance with the specified base path.
    ///
    /// The base path determines the prefix under which all routes registered
    /// to this router will be mounted when attached to an application.
    ///
    /// # Arguments
    ///
    /// * `base_path` - The base path (e.g., "/api" or "/v1") for this router.
    ///
    /// # Returns
    ///
    /// A new `Router` with an empty set of routes, ready for route registration.
    ///
    /// # Example
    ///
    /// ```
    /// use ripress::router::Router;
    /// let router = Router::new("/api");
    /// ```
    pub fn new(base_path: &'static str) -> Self {
        Router {
            base_path,
            routes: HashMap::new(),
        }
    }

    /// Registers a router with an app.
    ///
    /// ## Arguments
    ///
    /// * `mut app` - The instance of the app to register the router too
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::{router::Router, context::{HttpRequest, HttpResponse}, app::App};
    /// use crate::ripress::types::RouterFns;
    ///
    /// async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    ///     res.ok().text("Hello, World!")
    /// }
    ///
    /// let mut router = Router::new("/api");
    /// let mut app = App::new();
    /// router.patch("/hello", handler);
    /// router.register(&mut app);
    /// ```

    pub fn register(self, app: &mut App) {
        for (path, methods) in self.routes {
            for (method, handler) in methods {
                let full_path = format!("{}{}", self.base_path, path);
                app.add_route(method, &full_path, move |req, res| (handler)(req, res));
            }
        }
    }
}

impl RouterFns for Router {
    fn routes(&mut self) -> &mut Routes {
        &mut self.routes
    }
}
