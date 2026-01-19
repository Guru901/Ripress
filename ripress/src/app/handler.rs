use std::sync::Arc;

use crate::app::{api_error::ApiError, App, Http2Config};
use bytes::Bytes;
use http_body_util::Full;
use hyper::{server::conn::http1, service::Service};
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::{Builder, Http2Builder},
};
use routerify_ng::RouterService;

impl App {
    pub(crate) async fn handle_connection(
        stream: tokio::net::TcpStream,
        service: Arc<RouterService<ApiError>>,
        http2_enabled: bool,
        http2_config: Option<Http2Config>,
    ) {
        let request_service = match service.call(&stream).await {
            Ok(svc) => svc,
            Err(err) => {
                eprintln!("Error creating per-connection service: {:?}", err);
                return;
            }
        };

        let io = TokioIo::new(stream);

        if http2_enabled {
            if let Some(cfg) = http2_config {
                if cfg.http2_only {
                    Self::serve_http2_only(io, request_service, &cfg).await;
                } else {
                    Self::serve_http1_and_http2(io, request_service, &cfg).await;
                }
            } else {
                Self::serve_http1_and_http2_default(io, request_service).await;
            }
        } else {
            Self::serve_http1_only(io, request_service).await;
        }
    }

    async fn serve_http2_only<I, S>(io: I, service: S, cfg: &Http2Config)
    where
        I: hyper::rt::Read + hyper::rt::Write + Unpin + 'static,
        S: hyper::service::Service<
                hyper::Request<hyper::body::Incoming>,
                Response = hyper::Response<Full<Bytes>>,
            > + Send
            + 'static,
        S::Future: Send + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let mut builder = Builder::new(TokioExecutor::new());
        let mut h2: Http2Builder<'_, TokioExecutor> = builder.http2();

        Self::apply_http2_config(&mut h2, cfg);
        h2.enable_connect_protocol();

        if let Err(err) = h2.serve_connection(io, service).await {
            eprintln!("Error serving connection: {:?}", err);
        }
    }

    async fn serve_http1_and_http2<I, S>(io: I, service: S, cfg: &Http2Config)
    where
        I: hyper::rt::Read + hyper::rt::Write + Unpin + 'static + Send,
        S: hyper::service::Service<
                hyper::Request<hyper::body::Incoming>,
                Response = hyper::Response<Full<Bytes>>,
            > + Send
            + 'static,
        S::Future: Send + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let mut builder = Builder::new(TokioExecutor::new());

        builder.http1().keep_alive(true);

        let mut h2: Http2Builder<'_, TokioExecutor> = builder.http2();

        Self::apply_http2_config(&mut h2, cfg);

        h2.enable_connect_protocol();

        if let Err(err) = builder.serve_connection_with_upgrades(io, service).await {
            eprintln!("Error serving connection: {:?}", err);
        }
    }

    async fn serve_http1_and_http2_default<I, S>(io: I, service: S)
    where
        I: hyper::rt::Read + hyper::rt::Write + Unpin + 'static + Send,
        S: hyper::service::Service<
                hyper::Request<hyper::body::Incoming>,
                Response = hyper::Response<Full<Bytes>>,
            > + Send
            + 'static,
        S::Future: Send + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let mut builder = Builder::new(TokioExecutor::new());
        builder.http1().keep_alive(true);

        if let Err(err) = builder.serve_connection_with_upgrades(io, service).await {
            eprintln!("Error serving connection: {:?}", err);
        }
    }

    async fn serve_http1_only<I, S>(io: I, service: S)
    where
        I: hyper::rt::Read + hyper::rt::Write + Unpin + 'static + Send,
        S: hyper::service::Service<
                hyper::Request<hyper::body::Incoming>,
                Response = hyper::Response<Full<Bytes>>,
            > + Send
            + 'static,
        S::Future: Send + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let mut builder = http1::Builder::new();
        builder.keep_alive(true);

        if let Err(err) = builder.serve_connection(io, service).with_upgrades().await {
            eprintln!("Error serving connection: {:?}", err);
        }
    }

    fn apply_http2_config(h2: &mut Http2Builder<'_, TokioExecutor>, cfg: &Http2Config) {
        if let Some(v) = cfg.max_concurrent_streams {
            h2.max_concurrent_streams(v);
        }
        if let Some(v) = cfg.initial_stream_window_size {
            h2.initial_stream_window_size(v);
        }
        if let Some(v) = cfg.initial_connection_window_size {
            h2.initial_connection_window_size(v);
        }
        if let Some(v) = cfg.adaptive_window {
            h2.adaptive_window(v);
        }
        if let Some(v) = cfg.max_frame_size {
            h2.max_frame_size(v);
        }
        if let Some(v) = cfg.max_header_list_size {
            h2.max_header_list_size(v);
        }
        if let Some(v) = cfg.keep_alive_interval {
            h2.keep_alive_interval(v);
        }
        if let Some(v) = cfg.keep_alive_timeout {
            h2.keep_alive_timeout(v);
        }
    }
}
