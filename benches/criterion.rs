use std::{hint::black_box, process::Command};

use criterion::{Criterion, criterion_group, criterion_main};
use ripress::{app::App, types::RouterFns};

async fn bench_once() {
    for _ in 1..5000000 {
        let mut hyper_req: Request<Full<Bytes>> = Request::default();

        assert!(
            ripress::req::HttpRequest::from_hyper_request(&mut hyper_req)
                .await
                .is_ok()
        );
    }
}

fn bench_from_hyper(c: &mut Criterion) {
    c.bench_function("from_hyper_request", |b| {
        b.iter(|| async {
            for _ in 0..500_000 {
                black_box(bench_once().await);
            }
        })
    });
}

criterion_group!(benches, bench_from_hyper);
criterion_main!(benches);
