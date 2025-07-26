use crate::types::{RouterFns, Routes};
use std::collections::HashMap;

pub struct Router {
    base_path: String,
    routes: Routes,
}

impl Router {
    pub fn new() -> Self {
        Router {
            base_path: String::new(),
            routes: HashMap::new(),
        }
    }
}

impl RouterFns for Router {
    fn routes(&mut self) -> &mut Routes {
        &mut self.routes
    }
}
