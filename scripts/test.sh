#!/bin/bash
set -e  # Exit on error

cargo test --all  # Run Rust tests

cd src
touch main.rs

echo '

use ripress_again::app::Ripress;
use ripress_again::context::{HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[tokio::main]
async fn main() {
    let mut app = Ripress::new();

    // request tests
    app.get("/cookie-test", cookie_handler);
    app.get("/header-test", header_handler);
    app.get("/param-and-query-test/{param}", query_and_param_handler);
    app.get("/origin-url-and-path/test", path_and_origin_url_handler);
    app.get("/ip-test", ip_handler);
    app.post("/json-test", json_handler);
    app.post("/text-test", text_handler);
    app.post("/form-test", form_handler);

    // response tests

    app.get("/get-cookie-test", get_cookie_test);

    app.listen(8080, || {}).await.unwrap();
}

// requests test handler

async fn cookie_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let session_id = req.cookies.get("sessionId").unwrap();
    res.ok().json(json!({
        "sessionId": session_id
    }))
}

async fn header_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let header_value = req.get_header("Test-Header").unwrap();
    res.ok().json(json!({
        "header": header_value
    }))
}

async fn query_and_param_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let param = req.params.get("param").unwrap();
    let query = req.query_params.get("query").unwrap();

    res.ok().json(json!({
        "param": param,
        "query": query
    }))
}

async fn path_and_origin_url_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let origin_url = req.origin_url;
    let path = req.path;

    res.ok().json(json!({
        "originUrl": origin_url,
        "path": path
    }))
}

async fn ip_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let ip = req.ip;

    res.ok().json(json!({
        "ip": ip
    }))
}

#[derive(Serialize, Deserialize)]
struct JsonBody {
    name: String,
    age: i32,
}
async fn json_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let body = req.json::<JsonBody>().unwrap();

    res.ok().json(json!({
        "name":body.name,
        "age":body.age
    }))
}

async fn text_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let body = req.text().unwrap();

    res.ok().text(body)
}

async fn form_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let body = req.form_data().unwrap();
    let name = body.get("name").unwrap();

    res.ok().json(json!({
        "name": name
    }))
}

// response test handler

async fn get_cookie_test(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().set_cookie("test-cookie", "value").text("hehe")
}


' > main.rs

cargo run &  # Start server in background
SERVER_PID=$!  # Store server process ID

sleep 20

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
