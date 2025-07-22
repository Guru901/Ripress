use ripress_again::app::Ripress;

#[tokio::main]
async fn main() {
    let app = Ripress::new();

    app.get("/", |req, res| {});

    app.listen(3000, || println!("Server listening on port 3000"))
}
