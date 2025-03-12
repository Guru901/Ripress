# JSON Body example

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.post("/submit", submit_data);

    app.listen("127.0.0.1:3000").await;
}

async fn submit_data(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.json() {
        Ok(body) => res.status(200).json(json!({"message": "Data received", "data": body})),
        Err(_) => res.bad_request().json(json!({"error": "Invalid JSON"}))
    }
}

```

### Request

```bash
POST /submit
Content-Type: application/json

{
    "name": "Ripress",
    "version": "0.1.0"
}
```

### Response

```bash
{
    "message": "Data received",
    "data": {
        "name": "Ripress",
        "version": "0.1.0"
    }
}
```
