# URL Params example

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/user/{id}", get_user);

    app.listen("127.0.0.1:3000").await;
}

async fn get_user(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user_id = req.get_params("id").unwrap_or("unknown".to_string());
    res.status(200).text(format!("User ID: {user_id}"))
}
```

### Request

```bash
GET /user/123
```

### Response

```bash
User ID: 123
```
