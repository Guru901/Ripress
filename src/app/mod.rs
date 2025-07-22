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

    pub async fn listen<F: FnOnce()>(&self, port: u16, cb: F) -> std::io::Result<()> {
        cb();
        let routes = self.routes.clone();

        println!("{:?}", routes.len());

        actix_web::HttpServer::new(move || {
            routes
                .iter()
                .fold(actix_web::App::new(), |app, (path, (method, handler))| {
                    let route_method = match method {
                        HttpMethod::GET => actix_web::web::get(),
                        HttpMethod::POST => actix_web::web::post(),
                        HttpMethod::PUT => actix_web::web::put(),
                        HttpMethod::HEAD => actix_web::web::head(),
                    };

                    // Clone the handler to move it into the closure
                    let handler = handler.clone();
                    let path = path.clone();

                    app.route(
                        &path,
                        route_method.to(
                            move |req: actix_web::HttpRequest,
                                  _payload: actix_web::web::Payload| {
                                let handler = handler.clone();
                                async move {
                                    let our_req = HttpRequest::from_actix_request(req);
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
