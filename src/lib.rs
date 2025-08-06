/// The App Struct and it's methods.
pub mod app;

/// The Request Struct and it's methods.
pub mod req;

/// The Response Struct and it's methods.
pub mod res;

pub mod context {
    pub use super::req::HttpRequest;
    pub use super::res::HttpResponse;
}

pub mod middlewares;
pub mod router;
pub mod types;

mod tests;
