# Basic Routing example

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/", home);
    app.get("/about", about);

    app.listen(3000, || {}).await;
}

async fn home(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.status(200).text("Welcome to Ripress!")
}

async fn about(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.status(200)
        .text("Ripress is a lightweight web framework inspired by Express.js")
}
```
