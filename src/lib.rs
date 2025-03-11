use actix_web::Responder;
use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

pub type Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send + 'static>>;

fn box_future<F>(future: F) -> Fut
where
    F: Future<Output = HttpResponse> + Send + 'static,
{
    Box::pin(future)
}

pub type Handler = Arc<dyn Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static>;

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

pub struct HttpResponse {
    pub status_code: i32,
    pub body: String,
}

impl HttpResponse {
    pub fn new() -> Self {
        HttpResponse {
            status_code: 200,
            body: String::new(),
        }
    }

    pub fn status(mut self, code: i32) -> Self {
        self.status_code = code;
        return self;
    }

    pub fn json(mut self, json: serde_json::Value) -> Self {
        self.body = json.to_string();
        return self;
    }

    pub fn text(mut self, text: String) -> Self {
        self.body = text;
        return self;
    }

    fn to_responder(self) -> actix_web::HttpResponse {
        let body = self.body;
        actix_web::http::StatusCode::from_u16(self.status_code as u16)
            .map(|status| actix_web::HttpResponse::build(status).body(body))
            .unwrap_or_else(|_| {
                actix_web::HttpResponse::InternalServerError().body("Invalid status code")
            })
    }
}

impl Responder for HttpResponse {
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, req: &actix_web::HttpRequest) -> actix_web::HttpResponse {
        self.to_responder()
    }
}

#[derive(Debug)]
pub struct HttpRequest {
    params: HashMap<String, String>,
    queries: HashMap<String, String>,
}

impl HttpRequest {
    pub fn get_params(&self, param_name: &str) -> Option<String> {
        return Some(String::new());
    }

    pub fn get_query(&self, query_name: &str) -> Option<String> {
        return Some(String::new());
    }

    fn from_actix_request(req: &actix_web::HttpRequest) -> Self {
        HttpRequest {
            params: req
                .match_info()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            queries: HashMap::new(),
        }
    }
}

async fn index() -> impl actix_web::Responder {
    format!("Hello world!")
}
