use bytes::Bytes;
use criterion::{Criterion, criterion_group, criterion_main};
use http_body_util::Full;
use hyper::{Request, Response};
use ripress::{req::HttpRequest, res::HttpResponse};
use serde_json::json;
use std::{collections::HashMap, hint::black_box};
use tokio::runtime::Runtime;

fn create_test_hyper_request() -> Request<Full<Bytes>> {
    let mut hyper_req: Request<Full<Bytes>> = Request::default();

    {
        let headers = hyper_req.headers_mut();
        headers.insert("Foo", "Bar".parse().unwrap());
        headers.insert("X-Test-1", "Value1".parse().unwrap());
        headers.insert("X-Test-2", "Value2".parse().unwrap());
        headers.insert("X-Test-3", "Value3".parse().unwrap());
        headers.insert("X-Test-4", "Value4".parse().unwrap());
        headers.insert("X-Test-5", "Value5".parse().unwrap());
        headers.insert("X-Test-6", "Value6".parse().unwrap());
        headers.insert("X-Test-7", "Value7".parse().unwrap());
        headers.insert("X-Test-8", "Value8".parse().unwrap());
        headers.insert("X-Test-9", "Value9".parse().unwrap());

        let cookies = vec![
            "Foo=Bar",
            "Session=XYZ123",
            "hello=cookie",
            "Alpha=Beta",
            "User=Admin",
        ];
        let cookies_str = cookies.join("; ");
        headers.insert(hyper::header::COOKIE, cookies_str.parse().unwrap());
    }

    {
        let extensions = hyper_req.extensions_mut();
        let mut hashmap1 = HashMap::new();
        hashmap1.insert("Hello", "World");
        extensions.insert(hashmap1);

        let mut hashmap2 = HashMap::new();
        hashmap2.insert("Foo", "Bar");
        extensions.insert(hashmap2);

        let mut hashmap3 = HashMap::new();
        hashmap3.insert("Rust", "Lang");
        hashmap3.insert("Test", "Value");
        hashmap3.insert("One", "Two");
        extensions.insert(hashmap3);
    }

    hyper_req
}

fn create_test_custom_request() -> HttpRequest {
    let mut our_req = HttpRequest::new();
    // Fill it once
    our_req.headers.insert("Foo", "Bar");
    our_req.headers.insert("X-Test-1", "Value1");
    our_req.headers.insert("X-Test-2", "Value2");
    our_req.headers.insert("X-Test-3", "Value3");
    our_req.headers.insert("X-Test-4", "Value4");
    our_req.headers.insert("X-Test-5", "Value5");
    our_req.headers.insert("X-Test-6", "Value6");
    our_req.headers.insert("X-Test-7", "Value7");
    our_req.headers.insert("X-Test-8", "Value8");
    our_req.headers.insert("X-Test-9", "Value9");

    our_req.set_data("Test", "Value");
    our_req.set_data("One", "Two");

    our_req.set_cookie("Foo", "Bar");
    our_req.set_cookie("Session", "XYZ123");
    our_req.set_cookie("hello", "cookie");
    our_req.set_cookie("Alpha", "Beta");
    our_req.set_cookie("User", "Admin");

    our_req
}

fn create_test_custom_response() -> HttpResponse {
    let mut our_res = HttpResponse::new();
    // Fill it once
    our_res.headers.insert("Foo", "Bar");
    our_res.headers.insert("X-Test-1", "Value1");
    our_res.headers.insert("X-Test-2", "Value2");
    our_res.headers.insert("X-Test-3", "Value3");
    our_res.headers.insert("X-Test-4", "Value4");
    our_res.headers.insert("X-Test-5", "Value5");
    our_res.headers.insert("X-Test-6", "Value6");
    our_res.headers.insert("X-Test-7", "Value7");
    our_res.headers.insert("X-Test-8", "Value8");
    our_res.headers.insert("X-Test-9", "Value9");

    our_res = our_res.set_cookie("Test", "Value", None);
    our_res = our_res.set_cookie("One", "Two", None);
    our_res = our_res.set_cookie("Alpha", "Beta", None);
    our_res = our_res.set_cookie("User", "Admin", None);

    our_res = our_res.json(json!({
        "name": "nice",
        "name": "nice",
        "name": "nice",
        "name": "nice",
        "name": "nice",
        "name": "nice",
        "name": "nice",
    }));

    our_res = our_res.text("noiceh");
    our_res = our_res.bytes(Bytes::from("some data"));

    our_res
}

fn create_test_hyper_response() -> Response<Full<Bytes>> {
    let mut hyper_res: Response<Full<Bytes>> = Response::default();
    // Fill it once
    hyper_res
        .headers_mut()
        .insert("Foo", "Bar".parse().unwrap());
    hyper_res
        .headers_mut()
        .insert("X-Test-1", "Value1".parse().unwrap());
    hyper_res
        .headers_mut()
        .insert("X-Test-2", "Value2".parse().unwrap());
    hyper_res
        .headers_mut()
        .insert("X-Test-3", "Value3".parse().unwrap());
    hyper_res
        .headers_mut()
        .insert("X-Test-4", "Value4".parse().unwrap());
    hyper_res
        .headers_mut()
        .insert("X-Test-5", "Value5".parse().unwrap());
    hyper_res
        .headers_mut()
        .insert("X-Test-6", "Value6".parse().unwrap());
    hyper_res
        .headers_mut()
        .insert("X-Test-7", "Value7".parse().unwrap());
    hyper_res
        .headers_mut()
        .insert("X-Test-8", "Value8".parse().unwrap());
    hyper_res
        .headers_mut()
        .insert("X-Test-9", "Value9".parse().unwrap());

    hyper_res
        .headers_mut()
        .insert("set-cookie", "Cookie=Value".parse().unwrap());
    hyper_res
        .headers_mut()
        .insert("set-cookie", "Value".parse().unwrap());
    hyper_res
        .headers_mut()
        .insert("set-cookie", "Value".parse().unwrap());

    hyper_res
}

fn bench_from_hyper(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("to_hyper_request", |b| {
        b.iter(|| {
            let our_req = create_test_custom_request();

            rt.block_on(async {
                let req = black_box(our_req.to_hyper_request());
                black_box(req).unwrap()
            })
        })
    });

    c.bench_function("from_hyper_request", |b| {
        b.iter(|| {
            let mut hyper_req = create_test_hyper_request();

            rt.block_on(async {
                let req =
                    ripress::req::HttpRequest::from_hyper_request(black_box(&mut hyper_req)).await;
                black_box(req)
            })
        })
    });

    c.bench_function("to_hyper_response", |b| {
        b.iter(|| {
            let our_res = create_test_custom_response();

            rt.block_on(async {
                let hyper_res = black_box(our_res.to_hyper_response().await);
                black_box(hyper_res)
            })
        })
    });

    c.bench_function("from_hyper_response", |b| {
        b.iter(|| {
            let mut hyper_res = create_test_hyper_response();

            rt.block_on(async {
                let res =
                    ripress::res::HttpResponse::from_hyper_response(black_box(&mut hyper_res))
                        .await;
                black_box(res)
            })
        })
    });
}

criterion_group!(benches, bench_from_hyper);
criterion_main!(benches);
