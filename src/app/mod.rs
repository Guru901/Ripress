use crate::req::HttpRequest;
use crate::res::HttpResponse;
use crate::types::{Fut, HttpMethod, Routes};
use std::collections::HashMap;

pub(crate) fn box_future<F>(future: F) -> Fut
where
    F: Future<Output = HttpResponse> + Send + 'static,
{
    Box::pin(future)
}
use std::sync::Arc;

pub struct Ripress {
    routes: Routes,
}

impl Ripress {
    pub fn new() -> Self {
        Ripress {
            routes: HashMap::new(),
        }
    }

    fn add_route<F, Fut>(&mut self, method: HttpMethod, path: &str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        let wrapped_handler = Arc::new(move |req, res| box_future(handler(req, res)));
        self.routes
            .insert(path.to_string(), (method, wrapped_handler));
    }

    pub fn get<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        self.add_route(HttpMethod::GET, path, handler);
    }

    pub fn post<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        self.add_route(HttpMethod::POST, path, handler);
    }

    pub fn put<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        self.add_route(HttpMethod::PUT, path, handler);
    }

    pub fn delete<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        self.add_route(HttpMethod::DELETE, path, handler);
    }

    pub fn head<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        self.add_route(HttpMethod::HEAD, path, handler);
    }

    pub fn all<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + Clone + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        self.add_route(HttpMethod::GET, path, handler.clone());
        self.add_route(HttpMethod::POST, path, handler.clone());
        self.add_route(HttpMethod::PUT, path, handler.clone());
        self.add_route(HttpMethod::DELETE, path, handler.clone());
        self.add_route(HttpMethod::HEAD, path, handler);
    }

    pub async fn listen<F: FnOnce()>(&self, port: u16, cb: F) -> std::io::Result<()> {
        cb();
        let routes = self.routes.clone();

        actix_web::HttpServer::new(move || {
            routes
                .iter()
                .fold(actix_web::App::new(), |app, (path, (method, handler))| {
                    let route_method = match method {
                        HttpMethod::GET => actix_web::web::get(),
                        HttpMethod::POST => actix_web::web::post(),
                        HttpMethod::PUT => actix_web::web::put(),
                        HttpMethod::HEAD => actix_web::web::head(),
                        HttpMethod::DELETE => actix_web::web::delete(),
                    };

                    // Clone the handler to move it into the closure
                    let handler = handler.clone();
                    let path = path.clone();

                    app.route(
                        &path,
                        route_method.to(
                            move |req: actix_web::HttpRequest, payload: actix_web::web::Payload| {
                                let handler = handler.clone();
                                async move {
                                    let our_req = HttpRequest::from_actix_request(req, payload)
                                        .await
                                        .unwrap();
                                    let our_res = HttpResponse::new();
                                    let response = handler(our_req, our_res).await;
                                    response.to_responder()
                                }
                            },
                        ),
                    )
                })
        })
        .bind(format!("127.0.0.1:{}", port))?
        .run()
        .await
    }
}
