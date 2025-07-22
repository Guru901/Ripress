use ripress_again::app::Ripress;
use ripress_again::req::HttpRequest;
use ripress_again::res::HttpResponse;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut app = Ripress::new();

    app.get("/text", text_handler);

    app.listen(3000, || println!("Server listening on port 3000"))
        .await
}

async fn text_handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    return res.text("Hello, World!");
}
