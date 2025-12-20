/// A macro for convenient construction of middleware vectors (`Middlewares`).
///
/// # Usage
///
/// The `middlewares!` macro simplifies the creation of middleware lists by allowing you to specify
/// route patterns and corresponding middleware closures in a concise and readable way.
///
/// Each element is a tuple in the form: `("/path", |req, res| { ... })`.
///
/// # Example
///
/// ```rust
/// use ripress::{app::App, types::Middlewares, middlewares};
///
/// let pre_middlewares: Middlewares = middlewares![
///     ("/", |req, _res| Box::pin(async move { (req, None) })),
///     ("/admin", |req, _res| Box::pin(async move { (req, None) })),
/// ];
/// ```
///
/// # Output
///
/// Expands into a `Vec<(&'static str, Box<dyn Fn(...) -> ...>)>` ready for
/// use with `App::use_pre_middlewares()` or `App::use_post_middlewares()`.
#[macro_export]
macro_rules! middlewares {
    ( $( ($path:expr, $handler:expr) ),* $(,)? ) => {
        {
            let mut vec: $crate::types::Middlewares = Vec::new();
            $(
                vec.push((
                    $path,
                    Box::new($handler)
                ));
            )*
            vec
        }
    };
}
