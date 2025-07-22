use ripress_again::app::Ripress;
use serde_json::json;

#[tokio::main]
async fn main() {
    let app = Ripress::new();

    app.get("/text", |_req, res| {
        return res.text("Hello, World!");
    });

    app.get("/json", |_req, res| {
        let data = json!({
            "hehe": "hehe"
        });

        return res.json(data);
    });

    app.listen(3000, || println!("Server listening on port 3000"))
}
