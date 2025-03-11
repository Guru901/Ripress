use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use crate::request::HttpRequest;
use crate::response::HttpResponse;

type Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send + 'static>>;
type Handler = Arc<dyn Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static>;

fn box_future<F>(future: F) -> Fut
where
    F: Future<Output = HttpResponse> + Send + 'static,
{
    Box::pin(future)
}

type Routes = HashMap<&'static str, HashMap<HttpMethods, Handler>>;

#[derive(Eq, Hash, PartialEq, Clone)]
enum HttpMethods {
    GET,
    PUT,
    POST,
    DELETE,
}

pub struct App {
    routes: Routes,
}

impl Clone for App {
    fn clone(&self) -> Self {
        App {
            routes: self.routes.clone(),
        }
    }
}

impl App {
    pub fn new() -> App {
        return App {
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

    pub async fn listen(self, addr: &str) {
        println!("Server listening on {}", addr);

        let routes = self.routes.clone();

        actix_web::HttpServer::new(move || {
            let mut app = actix_web::App::new();

            for (path, methods) in &routes {
                for (method, handler) in methods {
                    let handler_clone = handler.clone();

                    match method {
                        HttpMethods::GET => {
                            app = app.route(
                                &path,
                                actix_web::web::get().to(move |req: actix_web::HttpRequest| {
                                    let our_req = HttpRequest::from_actix_request(&req);
                                    let our_res = HttpResponse::new();
                                    let future = handler_clone(our_req, our_res);
                                    async move {
                                        let response = future.await;
                                        response.to_responder()
                                    }
                                }),
                            );
                        }
                        HttpMethods::POST => {
                            app = app.route(
                                &path,
                                actix_web::web::post().to(move |req: actix_web::HttpRequest| {
                                    let our_req = HttpRequest::from_actix_request(&req);
                                    let our_res = HttpResponse::new();
                                    let future = handler_clone(our_req, our_res);
                                    async move {
                                        let response = future.await;
                                        response.to_responder()
                                    }
                                }),
                            );
                        }
                        HttpMethods::PUT => {
                            app = app.route(
                                &path,
                                actix_web::web::put().to(move |req: actix_web::HttpRequest| {
                                    let our_req = HttpRequest::from_actix_request(&req);
                                    let our_res = HttpResponse::new();
                                    let future = handler_clone(our_req, our_res);
                                    async move {
                                        let response = future.await;
                                        response.to_responder()
                                    }
                                }),
                            );
                        }
                        HttpMethods::DELETE => {
                            app = app.route(
                                &path,
                                actix_web::web::delete().to(move |req: actix_web::HttpRequest| {
                                    let our_req = HttpRequest::from_actix_request(&req);
                                    let our_res = HttpResponse::new();
                                    let future = handler_clone(our_req, our_res);
                                    async move {
                                        let response = future.await;
                                        response.to_responder()
                                    }
                                }),
                            );
                        }
                    }
                }
            }
            app
        })
        .bind(addr)
        .unwrap()
        .run()
        .await
        .unwrap();
    }

    fn add_route(&mut self, method: HttpMethods, path: &'static str, handler: Handler) {
        let path_handlers = self.routes.entry(path).or_insert_with(HashMap::new);
        path_handlers.insert(method, handler);
    }
}
