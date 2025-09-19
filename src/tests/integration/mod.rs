use crate::app::App;
use crate::middlewares::file_upload::file_upload;
use crate::types::RouterFns;
use crate::{context::HttpRequest, context::HttpResponse};
use bytes::Bytes;
use futures::stream;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use tokio::{task, time};

#[derive(Serialize, Deserialize)]
struct JsonBody {
    name: String,
    age: i32,
}

#[derive(Serialize, Deserialize)]
struct NoContentTypeData {
    test: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct LargeBodyData {
    data: String,
    numbers: Vec<i32>,
}

fn build_integration_app() -> App {
    let mut app = App::new();

    // Request routes
    app.get("/cookie-test", cookie_handler);
    app.get("/header-test", header_handler);
    app.get("/param-and-query-test/:param", query_and_param_handler);
    app.get("/origin-url-and-path/test", path_and_origin_url_handler);
    app.get("/ip-test", ip_handler);
    app.post("/json-test", json_handler);
    app.post("/text-test", text_handler);
    app.post("/form-test", form_handler);
    app.get("/multi-query", multi_query_handler);
    app.get("/multi-cookies", multi_cookie_handler);
    app.get("/multi-headers", multi_header_handler);
    app.get("/users/:user_id/posts/:post_id", multi_param_handler);
    app.get("/method-test", method_handler);
    app.post("/method-test", method_handler);
    app.put("/method-test", method_handler);
    app.delete("/method-test", method_handler);
    app.post("/urlencoded-test", urlencoded_handler);
    app.post("/raw-body-test", raw_body_handler);
    app.post("/empty-body-test", empty_body_handler);
    app.get("/xhr-test", xhr_handler);
    app.get("/secure-test", secure_handler);
    app.post("/json-error-test", json_error_test);
    app.get("/long-header-test", extremely_long_header_test);
    app.post("/no-content-type-test", no_content_type_test);
    app.post("/content-type-mismatch-test", content_type_mismatch_test);
    app.get("/special-query-test", special_query_test);
    app.post("/large-body-test", large_body_test);
    app.use_pre_middleware("/multipart-file-test", file_upload(None));
    app.post("/multipart-text-test", multipart_text_test);
    app.post("/multipart-file-test", multipart_file_test);

    // Auth
    app.use_pre_middleware("/auth", |req, res| {
        let has_token = req.get_cookie("token").is_some();
        let req_cloned = req.clone();
        Box::pin(async move {
            if has_token {
                (req_cloned, None)
            } else {
                (req_cloned, Some(res.unauthorized().text("unauthorized")))
            }
        })
    });
    app.get("/auth", auth);

    // Response routes
    app.get("/get-cookie-test", get_cookie_test);
    app.get("/multiple-cookies-test", get_multi_cookie_test);
    app.get("/cookie-options-test", get_cookie_with_options_test);
    app.get("/custom-headers-test", custom_headers_test);
    app.post("/created-test", created_test);
    app.get("/custom-status-test", custom_status_test);
    app.get("/redirect-test", redirect_test);
    app.get("/permanent-redirect-test", permanent_redirect_test);
    app.get("/binary-test", binary_test);
    app.get("/cors-test", cors_test);
    app.get("/multiple-headers-test", multiple_headers_test);
    app.delete("/no-content-test", no_content_test);

    // Streaming routes
    app.get("/stream-text", stream_text);
    app.get("/stream-json", stream_json);

    // Static files mount for tests will be set by the test itself using a temp dir

    app
}

pub(super) async fn spawn_test_server(port: u16) -> task::JoinHandle<()> {
    let app = build_integration_app();
    task::spawn(async move { app.listen(port, || {}).await })
}

async fn cookie_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let session_id = req.get_cookie("sessionId").unwrap();
    res.ok().json(json!({ "sessionId": session_id }))
}

async fn header_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let header_value = req.headers.get("Test-Header").unwrap();
    res.ok().json(json!({ "header": header_value }))
}

async fn query_and_param_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let param = req.params.get("param").unwrap();
    let query = req.query.get("query").unwrap();
    res.ok().json(json!({ "param": param, "query": query }))
}

async fn path_and_origin_url_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let origin_url = req.origin_url.to_string();
    let path = req.path;
    res.ok()
        .json(json!({ "originUrl": origin_url, "path": path }))
}

async fn ip_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let ip = req.ip;
    res.ok().json(json!({ "ip": ip }))
}

async fn json_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let body = req.json::<JsonBody>().unwrap();
    res.ok().json(json!({ "name": body.name, "age": body.age }))
}

async fn text_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text(req.text().unwrap())
}

async fn form_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let body = req.form_data().unwrap();
    let name = body.get("name").unwrap();
    res.ok().json(json!({ "name": name }))
}

async fn multi_query_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let name = req.query.get("name").unwrap();
    let age = req.query.get("age").unwrap();
    let city = req.query.get("city").unwrap();
    res.ok()
        .json(json!({ "name": name, "age": age, "city": city }))
}

async fn multi_param_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user_id = req.params.get("user_id").unwrap();
    let post_id = req.params.get("post_id").unwrap();
    res.ok()
        .json(json!({ "userId": user_id, "postId": post_id }))
}

async fn multi_cookie_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user = req.get_cookie("user").unwrap();
    let theme = req.get_cookie("theme").unwrap();
    let lang = req.get_cookie("lang").unwrap();
    res.ok()
        .json(json!({ "user": user, "theme": theme, "lang": lang }))
}

async fn multi_header_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user_agent = req.headers.get("User-Agent").unwrap();
    let accept = req.headers.get("Accept").unwrap();
    let auth = req.headers.get("Authorization").unwrap();
    let custom_header = req.headers.get("X-Custom-Header").unwrap();
    res.ok().json(json!({ "userAgent": user_agent, "accept": accept, "authorization": auth, "customHeader": custom_header }))
}

async fn method_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().json(json!(req.method.to_string()))
}

async fn urlencoded_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let form_data = req.form_data().unwrap();
    let username = form_data.get("username").unwrap();
    let password = form_data.get("password").unwrap();
    let email = form_data.get("email").unwrap();
    res.ok()
        .json(json!({ "username": username, "password": password, "email": email }))
}

async fn raw_body_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let body = req.text().unwrap();
    let content_type = req.headers.get("content-type").unwrap();
    res.ok()
        .json(json!({ "rawBody": body, "contentType": content_type }))
}

async fn empty_body_handler(_: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().json(json!({}))
}

async fn xhr_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().json(json!({ "xhr": req.xhr }))
}

async fn secure_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().json(json!(req.is_secure))
}

async fn json_error_test(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.json::<JsonBody>() {
        Ok(body) => res.ok().json(json!({ "name": body.name, "age": body.age })),
        Err(_) => res
            .status(400)
            .json(json!({ "error": "Validation failed" })),
    }
}

async fn extremely_long_header_test(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let header = req.headers.get("X-Long-Header").unwrap();
    res.ok().json(json!({ "header": header }))
}

async fn no_content_type_test(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.json::<NoContentTypeData>() {
        Ok(_) => res.ok(),
        Err(_) => res.status(400),
    }
}

async fn content_type_mismatch_test(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    if let Ok(data) = req.json::<NoContentTypeData>() {
        res.ok().json(json!({ "data": data.test }))
    } else {
        res.ok().text(req.text().unwrap())
    }
}

async fn special_query_test(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let name = req.query.get("name").unwrap();
    let symbols = req.query.get("symbols").unwrap();
    let unicode = req.query.get("unicode").unwrap();
    res.ok()
        .json(json!({ "name": name, "symbols": symbols, "unicode": unicode }))
}

async fn large_body_test(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let data = req.json::<LargeBodyData>().unwrap();
    res.ok().json(json!(data))
}

async fn multipart_text_test(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let body = req.form_data().unwrap();
    let name = body.get("name").unwrap();
    let email = body.get("email").unwrap();
    let age = body.get("age").unwrap();
    let description = body.get("description").unwrap();
    res.ok()
        .json(json!({ "name": name, "email": email, "age": age, "description": description }))
}

async fn multipart_file_test(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let _ = req.get_all_data();
    res.ok()
}

async fn get_cookie_test(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().set_cookie("test-cookie", "value", None)
}

async fn binary_test(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.bytes(vec![1, 2, 3, 4, 5])
}

async fn get_cookie_with_options_test(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    use crate::res::response_cookie::{CookieOptions, CookieSameSiteOptions};
    res.ok().set_cookie(
        "secure-cookie",
        "value",
        Some(CookieOptions {
            http_only: true,
            same_site: CookieSameSiteOptions::Strict,
            secure: true,
            ..Default::default()
        }),
    )
}

async fn get_multi_cookie_test(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok()
        .set_cookie("session", "abc123", None)
        .set_cookie("theme", "dark", None)
        .set_cookie("lang", "en", None)
}

async fn custom_headers_test(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok()
        .set_header("x-custom-header", "custom-value")
        .set_header("x-api-version", "1.0")
        .set_header("x-powered-by", "Ripress")
}

async fn created_test(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.status(201).json(json!({ "created": true }))
}

async fn custom_status_test(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.status(418).text("I'm a teapot")
}

async fn redirect_test(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.redirect("/redirected")
}

async fn permanent_redirect_test(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.permanent_redirect("/new-location")
}

async fn auth(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.get_cookie("token") {
        Some(token) => res.ok().text(token),
        None => res.unauthorized().text("unauthorized"),
    }
}

async fn stream_text(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let s = stream::unfold(1, |state| async move {
        if state <= 3 {
            time::sleep(Duration::from_millis(10)).await;
            Some((
                Ok::<Bytes, std::io::Error>(Bytes::from(format!("chunk{}\n", state))),
                state + 1,
            ))
        } else {
            None
        }
    });
    res.write(s)
}

async fn stream_json(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let s = stream::unfold(1, |state| async move {
        if state <= 3 {
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
    res.write(s)
}

async fn cors_test(_: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok()
}

async fn multiple_headers_test(_: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok()
        .set_header("x-header-1", "value1")
        .set_header("x-header-2", "value2")
        .set_header("x-header-3", "value3")
}

async fn no_content_test(_: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.no_content()
}
