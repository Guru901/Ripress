#!/bin/bash
set -e  # Exit on error

cargo test --all  # Run Rust tests

mkdir -p public/assets public/scripts
cd public
touch index.html app.js config.json readme.txt unknown-file.asdxyz styles.css
curl -sL https://upload.wikimedia.org/wikipedia/commons/6/6a/JavaScript-logo.png -o logo.png
curl -sL https://upload.wikimedia.org/wikipedia/commons/9/99/Sample_User_Icon.png -o photo.jpg
touch assets/styles.css scripts/app.js

echo '{}' > config.json
echo 'body {
    margin: 0;
}' > assets/styles.css
echo 'body {
    margin: 0;
}' > styles.css
echo 'function main() {
  document.querySelector("main");
}' > app.js
echo 'function main() {
  document.querySelector("main");
}' > scripts/app.js
echo '<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Document</title>
  </head>
  <body></body>
</html>' > index.html
echo 'This is a readme file' > readme.txt

cd ../src
touch main.rs

echo '

use bytes::Bytes;
use futures::stream;
use ripress::app::App;
use ripress::context::{HttpRequest, HttpResponse};
use ripress::middlewares::logger::logger;
use ripress::res::{CookieOptions, CookieSameSiteOptions};
use ripress::types::RouterFns;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.use_middleware("/", logger(None));

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
    app.post("/empty-body-test", empty_body_handler);

    app.get("/xhr-test", xhr_handler);
    app.get("/secure-test", secure_handler);
    app.get("/custom-headers-test", custom_headers_test);
    app.post("/created-test", created_test);
    app.get("/custom-status-test", custom_status_test);
    app.get("/redirect-test", redirect_test);
    app.get("/permanent-redirect-test", permanent_redirect_test);

    app.use_middleware("/auth", |req, res, next| {
        println!("Auth middleware");
        Box::pin(async move {
            match req.get_cookie("token") {
                Ok(_) => next.run(req, res).await,
                Err(_) => res.unauthorized().text("Unauthorized"),
            }
        })
    });

    app.get("/auth", auth);

    // response tests
    app.get("/get-cookie-test", get_cookie_test);
    app.get("/multiple-cookies-test", get_multi_cookie_test);
    app.get("/cookie-options-test", get_cookie_with_options_test);

    // streaming tests
    app.get("/stream-text", stream_text);
    app.get("/stream-json", stream_json);

    // Static Files test

    app.static_files("/static", "../public");

    app.listen(8080, || println!("Server is running on port 8080"))
        .await
        .unwrap();
}

// requests test handler

async fn cookie_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let session_id = req.get_cookie("sessionId").unwrap();
    res.ok().json(json!({
        "sessionId": session_id
    }))
}

async fn header_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let header_value = req.headers.get("Test-Header").unwrap();
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
    let origin_url = req.origin_url.to_string();
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
    let name = req.query_params.get("name").unwrap();
    let age = req.query_params.get("age").unwrap();
    let city = req.query_params.get("city").unwrap();

    res.ok().json(json!({
        "name": name,
        "age": age,
        "city": city,
    }))
}

async fn multi_param_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user_id = req.params.get("user_id").unwrap();
    let post_id = req.params.get("post_id").unwrap();

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
    let user_agent = req.headers.get("User-Agent").unwrap();
    let accept = req.headers.get("Accept").unwrap();
    let auth = req.headers.get("Authorization").unwrap();
    let custom_header = req.headers.get("X-Custom-Header").unwrap();

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
    let content_type = req.headers.get("content-type").unwrap();

    res.ok().json(json!({
        "rawBody": body,
        "contentType": content_type
    }))
}

async fn empty_body_handler(_: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().json(json!({}))
}

async fn xhr_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let xhr = req.xhr;
    res.ok().json(json!(xhr))
}
async fn secure_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let secure = req.is_secure;
    res.ok().json(json!(secure))
}

// response test handler

async fn get_cookie_test(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok()
        .set_cookie("test-cookie", "value", CookieOptions::default())
}

async fn get_cookie_with_options_test(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().set_cookie(
        "secure-cookie",
        "value",
        CookieOptions {
            http_only: true,
            same_site: CookieSameSiteOptions::Strict,
            secure: true,
            ..Default::default()
        },
    )
}

async fn get_multi_cookie_test(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok()
        .set_cookie("session", "abc123", CookieOptions::default())
        .set_cookie("theme", "dark", CookieOptions::default())
        .set_cookie("lang", "en", CookieOptions::default())
}

async fn custom_headers_test(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok()
        .set_header("x-custom-header", "custom-value")
        .set_header("x-api-version", "1.0")
        .set_header("x-powered-by", "Ripress")
}

async fn created_test(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.status(201).json(json!({
        "created": true
    }))
}

async fn custom_status_test(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.status(418).json(json!({
        "statusText": "I m a teapot"
    }))
}

async fn redirect_test(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.redirect("/redirected")
}

async fn permanent_redirect_test(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.permanent_redirect("/new-location")
}

async fn auth(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.get_cookie("token") {
        Ok(token) => res.ok().text(token),
        Err(_) => res.unauthorized().text("No token found"),
    }
}

async fn stream_text(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let stream = stream::unfold(1, |state| async move {
        if state < 20 {
            time::sleep(Duration::from_millis(10)).await;
            Some((
                Ok::<Bytes, std::io::Error>(Bytes::from(format!("chunk{}\n", state))),
                state + 1,
            ))
        } else {
            None
        }
    });

    res.write(stream)
}

async fn stream_json(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let stream = stream::unfold(1, |state| async move {
        if state < 10 {
            time::sleep(Duration::from_millis(10)).await;
            Some((
                Ok::<Bytes, std::io::Error>(Bytes::from(
                    serde_json::to_vec(&json!({ "id": state })).unwrap(),
                )),
                state + 1,
            ))
        } else {
            None
        }
    });

    res.write(stream)
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
cd ..
rm -rf public

echo "All Tests passed!"