#!/bin/bash
set -e  # Exit on error

cargo test --all  # Run Rust tests

cd src
touch main.rs

echo '
use ripress::app::App;
use ripress::context::{HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    // request tests
    app.get("/cookie-test", cookie_handler);
    app.get("/header-test", header_handler);
    app.get("/param-and-query-test/{param}", query_and_param_handler);
    app.get("/origin-url-and-path/test", path_and_origin_url_handler);
    app.get("/ip-test", ip_handler);
    app.post("/json-test", json_handler);
    app.post("/text-test", text_handler);
    app.post("/form-test", form_handler);
    app.get("/multi-query", multi_query_handler);
    app.get("/multi-cookies", multi_cookie_handler);
    app.get("/multi-headers", multi_header_handler);
    app.get("/users/{user_id}/posts/{post_id}", multi_param_handler);

    app.get("/method-test", method_handler);
    app.post("/method-test", method_handler);
    app.put("/method-test", method_handler);
    app.delete("/method-test", method_handler);

    app.post("/urlencoded-test", urlencoded_handler);
    app.post("/raw-body-test", raw_body_handler);

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

    // response tests

    app.get("/get-cookie-test", get_cookie_test);

    app.listen(8080, || {}).await.unwrap();
}

// requests test handler

async fn cookie_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let session_id = req.get_cookie("sessionId").unwrap();
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
    let param = req.get_params("param").unwrap();
    let query = req.get_query("query").unwrap();

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

async fn multi_query_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let name = req.get_query("name").unwrap();
    let age = req.get_query("age").unwrap();
    let city = req.get_query("city").unwrap();

    res.ok().json(json!({
        "name": name,
        "age": age,
        "city": city,
    }))
}

async fn multi_param_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user_id = req.get_params("user_id").unwrap();
    let post_id = req.get_params("post_id").unwrap();

    res.ok().json(json!({
        "userId": user_id,
        "postId": post_id
    }))
}

async fn multi_cookie_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user = req.get_cookie("user").unwrap();
    let theme = req.get_cookie("theme").unwrap();
    let lang = req.get_cookie("lang").unwrap();

    res.ok().json(json!({
        "user": user,
        "theme": theme,
        "lang": lang,
    }))
}

async fn multi_header_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user_agent = req.get_header("User-Agent").unwrap();
    let accept = req.get_header("Accept").unwrap();
    let auth = req.get_header("Authorization").unwrap();
    let custom_header = req.get_header("X-Custom-Header").unwrap();

    res.ok().json(json!({
        "userAgent": user_agent,
        "accept": accept,
        "authorization": auth,
        "customHeader": custom_header
    }))
}

async fn method_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let method = req.method;
    res.ok().json(json!(method.to_string()))
}

async fn urlencoded_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let form_data = req.form_data().unwrap();
    let username = form_data.get("username").unwrap();
    let password = form_data.get("password").unwrap();
    let email = form_data.get("email").unwrap();

    res.ok().json(json!({
        "username": username,
        "password": password,
        "email": email
    }))
}

async fn raw_body_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let body = req.text().unwrap();
    let content_type = req.get_header("content-type").unwrap();

    res.ok().json(json!({
        "rawBody": body,
        "contentType": content_type
    }))
}

// response test handler

async fn get_cookie_test(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().set_cookie("test-cookie", "value").text("hehe")
}

async fn auth(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let token = req.get_data("token").unwrap();
    res.ok().text(token)
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
