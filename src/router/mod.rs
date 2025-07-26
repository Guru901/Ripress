use crate::{
    app::App,
    types::{RouterFns, Routes},
};
use std::collections::HashMap;

pub struct Router {
    /// Base Path on which the router will be mounted to the app
    base_path: String,

    /// Routes registered on the router
    routes: Routes,
}

impl Router {
    pub fn new(base_path: &str) -> Self {
        Router {
            base_path: base_path.to_owned(),
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
