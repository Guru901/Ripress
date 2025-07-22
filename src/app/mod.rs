use crate::req::HttpRequest;
use crate::res::HttpResponse;

pub struct Ripress {
    port: u16,
}

impl Ripress {
    pub fn new() -> Self {
        Ripress { port: 0 }
    }

    pub fn get<F: Fn(HttpRequest, HttpResponse) -> HttpResponse>(&self, path: &str, handler: F) {}

    pub fn listen<F: FnOnce()>(&self, port: u16, cb: F) {
        cb()
    }
}
