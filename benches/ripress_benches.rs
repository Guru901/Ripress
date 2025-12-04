use std::hint::black_box;

use bytes::Bytes;
use criterion::{Criterion, criterion_group, criterion_main};
use http_body_util::Full;
use hyper::{Request, Response};
use ripress::{context::HttpRequest, res::HttpResponse};

/// Build a simple Hyper request with a JSON body.
fn build_json_request() -> Request<Full<Bytes>> {
    let body = serde_json::json!({
        "message": "hello",
        "value": 42,
    });
    let bytes = Bytes::from(serde_json::to_vec(&body).unwrap());

    Request::builder()
        .method("POST")
        .uri("http://localhost:3000/api/test?foo=bar&baz=qux")
        .header("host", "localhost:3000")
        .header("content-type", "application/json")
        .header("x-forwarded-for", "192.168.0.1")
        .header("x-forwarded-proto", "http")
        .header("x-requested-with", "XMLHttpRequest")
        .header("cookie", "session=abc123; user=demo")
        .body(Full::from(bytes))
        .unwrap()
}

/// Build a simple Hyper request with a text body.
fn build_text_request() -> Request<Full<Bytes>> {
    let body = Bytes::from("Hello from Ripress".to_owned());

    Request::builder()
        .method("GET")
        .uri("http://localhost:3000/text")
        .header("host", "localhost:3000")
        .header("content-type", "text/plain; charset=utf-8")
        .header("x-forwarded-for", "10.0.0.1")
        .header("x-forwarded-proto", "https")
        .body(Full::from(body))
        .unwrap()
}

/// Build a Hyper response with a JSON body.
fn build_json_response() -> Response<Full<Bytes>> {
    let body = serde_json::json!({
        "ok": true,
        "items": [1, 2, 3, 4, 5],
    });
    let bytes = Bytes::from(serde_json::to_vec(&body).unwrap());

    Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .header("x-powered-by", "ripress-bench")
        .body(Full::from(bytes))
        .unwrap()
}

/// Build a Hyper response with a text body.
fn build_text_response() -> Response<Full<Bytes>> {
    let body = Bytes::from("Hello from Hyper".to_owned());

    Response::builder()
        .status(200)
        .header("content-type", "text/plain; charset=utf-8")
        .body(Full::from(body))
        .unwrap()
}

fn bench_request_from_hyper_json(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("request_from_hyper_json", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut req = build_json_request();
                let _rip = HttpRequest::from_hyper_request(black_box(&mut req))
                    .await
                    .unwrap();
                black_box(_rip);
            })
        })
    });
}

fn bench_request_from_hyper_text(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("request_from_hyper_text", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut req = build_text_request();
                let _rip = HttpRequest::from_hyper_request(black_box(&mut req))
                    .await
                    .unwrap();
                black_box(_rip);
            })
        });
    });
}

fn bench_request_to_hyper_json(c: &mut Criterion) {
    // Build a Ripress HttpRequest from a Hyper request once, then
    // benchmark only the conversion back to Hyper.
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut base_req = build_json_request();
    let http_req = rt
        .block_on(HttpRequest::from_hyper_request(&mut base_req))
        .unwrap();

    c.bench_function("request_to_hyper_json", |b| {
        b.iter(|| {
            // Clone so each iteration starts from the same state
            let r = http_req.clone();
            let _hyper_req = r.to_hyper_request().unwrap();
            black_box(_hyper_req);
        })
    });
}

fn bench_response_from_hyper_json(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("response_from_hyper_json", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut res = build_json_response();
                let rip_res = HttpResponse::from_hyper_response(black_box(&mut res))
                    .await
                    .unwrap();
                black_box(rip_res);
            })
        });
    });
}

fn bench_response_from_hyper_text(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("response_from_hyper_text", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut res = build_text_response();
                let rip_res = HttpResponse::from_hyper_response(black_box(&mut res))
                    .await
                    .unwrap();
                black_box(rip_res);
            })
        })
    });
}

fn bench_response_to_hyper_json(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut base_res = build_json_response();
    let http_res = rt
        .block_on(HttpResponse::from_hyper_response(&mut base_res))
        .unwrap();

    c.bench_function("response_to_hyper_json", |b| {
        b.iter(|| {
            rt.block_on(async {
                let res = http_res.clone();
                let hyper_res: Response<Full<Bytes>> = res.to_hyper_response().await.unwrap();
                black_box(hyper_res);
            })
        });
    });
}

fn bench_roundtrip_full(c: &mut Criterion) {
    // Full roundtrip: Hyper Request -> Ripress -> Hyper Request
    // and Hyper Response -> Ripress -> Hyper Response
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("roundtrip_request_response", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Request roundtrip
                let mut req = build_json_request();
                let rip_req = HttpRequest::from_hyper_request(&mut req).await.unwrap();
                let hyper_req_back = rip_req.to_hyper_request().unwrap();

                // Response roundtrip
                let mut res = build_json_response();
                let rip_res = HttpResponse::from_hyper_response(&mut res).await.unwrap();
                let hyper_res_back: Response<Full<Bytes>> =
                    rip_res.to_hyper_response().await.unwrap();

                black_box((hyper_req_back, hyper_res_back));
            })
        });
    });
}

fn criterion_benches(c: &mut Criterion) {
    bench_request_from_hyper_json(c);
    bench_request_from_hyper_text(c);
    bench_request_to_hyper_json(c);
    bench_response_from_hyper_json(c);
    bench_response_from_hyper_text(c);
    bench_response_to_hyper_json(c);
    bench_roundtrip_full(c);
}

criterion_group!(benches, criterion_benches);
criterion_main!(benches);
