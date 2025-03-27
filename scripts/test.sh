#!/bin/bash
set -e  # Exit on error

cargo test --all  # Run Rust tests

cd src
touch main.rs

echo '
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
};
use serde_json::json;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/cookie-test", cookie_test);
    app.get("/header-test", header_test);
    app.get("/param-and-query-test/{param}", param_and_query_test);
    app.get("/origin-url-and-path/{param}", origin_url_and_path_test);
    app.get("/ip-test", ip_test);

    app.post("/json-test", json_test);
    app.post("/text-test", text_test);
    app.post("/form-test", form_test);

    app.get("/check-status-code", check_status_code);

    app.get("/get-cookie-test", get_cookie_test);

    app.use_middleware("/auth", |mut req, res, next| {
        println!("Auth middleware");
        Box::pin(async move {
            if let Ok(token) = req.get_cookie("token") {
                let token = token.to_string();
                req.set_data("token", &token);
                next.run(req, res).await
            } else {
                res.status(401).text("Unauthorized")
            }
        })
    });

    app.get("/auth", auth);

    app.listen(8080, || {
        println!("Serer running on port 8080");
    }).await;
}

async fn cookie_test(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let session_id = req.get_cookie("sessionId").unwrap();
    res.ok().json(json!({ "sessionId": session_id }))
}

async fn header_test(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let header = req.get_header("Test-header").unwrap();
    res.ok().json(json!({"header": header}))
}

async fn param_and_query_test(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let param = req.get_params("param").unwrap();
    let query = req.get_query("query").unwrap();
    res.ok().json(json!({"param": param, "query": query}))
}

async fn origin_url_and_path_test(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let origin_url = req.get_origin_url().unwrap();
    let path = req.get_path();
    res.ok()
        .json(json!({"originUrl": origin_url, "path": path}))
}

async fn ip_test(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let ip = req.ip().unwrap();
    res.ok().json(json!({"ip": ip}))
}

async fn json_test(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    #[derive(serde::Deserialize, serde::Serialize)]
    struct MyStruct {
        name: String,
        age: u8,
    }
    let body = req.json::<MyStruct>().unwrap();
    res.ok().json(body)
}

async fn text_test(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let body = req.text().unwrap();
    res.ok().text(body)
}

async fn form_test(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let body = req.form_data().unwrap();
    res.ok().json(body.get("name").unwrap())
}

async fn check_status_code(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.status(900)
}

async fn get_cookie_test(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().set_cookie("test-cookie", "value").text("hehe")
}

async fn auth(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let token = req.get_data("token").unwrap();
    res.ok().text(token)
}
' > main.rs

cargo run &  # Start server in background
SERVER_PID=$!  # Store server process ID

# Wait for the server to be ready
sleep 10

cd ../tests
bun install

# Run Playwright tests, fail script if tests fail
bunx playwright test || { 
  echo "Playwright tests failed"
  kill $SERVER_PID
  exit 1
}

kill $SERVER_PID  # Stop the server

cd ../src
rm main.rs
