use std::{collections::HashMap, future::Future, sync::Arc};

use crate::{
    app::{box_future, App},
    context::{HttpRequest, HttpResponse},
    types::{Handler, HttpMethods, Routes},
};

pub struct Router {
    base_path: String,
    routes: Routes,
}

impl Router {
    pub fn new(base_path: &str) -> Router {
        return Router {
            base_path: base_path.to_string(),
            routes: HashMap::new(),
        };
    }

    pub fn get<F, Fut>(&mut self, path: &'static str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        let wrapped_handler = Arc::new(move |req, res| box_future(handler(req, res)));
        self.add_route(HttpMethods::GET, path, wrapped_handler);
    }

    pub fn post<F, Fut>(&mut self, path: &'static str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        let wrapped_handler = Arc::new(move |req, res| box_future(handler(req, res)));
        self.add_route(HttpMethods::POST, path, wrapped_handler);
    }

    pub fn put<F, Fut>(&mut self, path: &'static str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        let wrapped_handler = Arc::new(move |req, res| box_future(handler(req, res)));
        self.add_route(HttpMethods::PUT, path, wrapped_handler);
    }

    pub fn delete<F, Fut>(&mut self, path: &'static str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        let wrapped_handler = Arc::new(move |req, res| box_future(handler(req, res)));
        self.add_route(HttpMethods::DELETE, path, wrapped_handler);
    }

    pub fn patch<F, Fut>(&mut self, path: &'static str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        let wrapped_handler = Arc::new(move |req, res| box_future(handler(req, res)));
        self.add_route(HttpMethods::PATCH, path, wrapped_handler);
    }

    pub fn register(self, app: &mut App) {
        for (path, methods) in self.routes.clone() {
            for (method, handler) in methods {
                let full_path = format!("{}{}", self.base_path, path);
                let path_str = Box::<str>::leak(full_path.into_boxed_str());
                app.add_route(method, path_str, handler);
            }
        }
    }

    fn add_route(&mut self, method: HttpMethods, path: &'static str, handler: Handler) {
        let path_handlers = self.routes.entry(path).or_insert_with(HashMap::new);
        path_handlers.insert(method, handler);
    }
}

#[cfg(test)]
impl Router {
    pub(crate) fn get_routes(&self, path: &str, method: HttpMethods) -> Option<&Handler> {
        Some(self.routes.get(path).unwrap().get(&method).unwrap())
    }
}
