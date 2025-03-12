# Form Data example

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.post("/save-user", save_user);

    app.listen("127.0.0.1:3000").await;
}

async fn save_user(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.form_data() {
        Ok(form) => {
            let user = form.get("user").unwrap_or("unknown".to_string());
            // save user here
            res.status(200).json(json!({"message": "Form received", "user": user}))
        }
        Err(_) => res.status(400).json(json!({"error": "Invalid FormData"}))
    }
}
```

### Request

```bash
Key: user → Value: JohnDoe
```

### Response

```bash
{
    "message": "Form received",
    "user": "JohnDoe"
}
```
