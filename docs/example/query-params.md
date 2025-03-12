# Query Params example

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/search", search);

    app.listen("127.0.0.1:3000").await;
}

async fn search(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let query = req.get_query("q").unwrap_or("nothing".to_string());
    res.status(200).text(format!("You searched for: {query}"))
}
```

### Request

```bash
GET /search?q=Ripress
```

### Response

```bash
You searched for: Ripress
```
