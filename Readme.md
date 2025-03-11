# Ripress

### An express inspired rust based web framework

#### NOTE: This still is an experiment don't think i will be able to complete it

### What am i making

- So it's basically an http server
- Written in rust
- Inspired by express
- First throwaway version will be built on top of actix web and then will see

### What are my goals for the project

- I want the end user experience to be simple and intuitive like in express
- I don't care much about performance in the starting as no matter how shitty my code will be it will be faster than actual express in typescript so, yeah

### What will the throwaway version have

- Only focused on routing different types of requests no middleware support

### Public Api

```rust
use rustyX::{App, HttpRequest, HttpResponse};
use serde_json::json;

#[tokio::main]
async fn main() {
    let app = App::new();

    app.get("/", index);
    app.get("/user/:id", find_user);
    app.get("/search", search);

    app.listen("127.0.0.1:3000");
}

async fn index(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    return res.status(200).json(json!({
        "hehehe": "hehehe"
    }));
}

async fn find_user(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user_id = req.get_params("id").unwrap_or(String::new());
    return res.status(200).text(format!("Hello, {user_id}"));
}

async fn search(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let q = req.get_query("q").unwrap_or(String::new());
    return res.status(200).text(format!("Nothing found for search: {q}"));
}
```
