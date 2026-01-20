#![warn(missing_docs)]
use crate::{
    context::HttpResponse,
    req::HttpRequest,
    types::{FutMiddleware, ResponseContentBody},
};
use flate2::{Compression, write::GzEncoder};
use std::io::Write;

/// Configuration for the compression middleware
#[derive(Clone)]
pub struct CompressionConfig {
    /// Minimum response size threshold to trigger compression (in bytes)
    pub threshold: usize,
    /// Compression level (0-9, where 6 is default, 9 is maximum compression)
    pub level: u8,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            threshold: 1024, 
            level: 6,
        }
    }
}

/// Creates a compression middleware that gzip compresses response bodies
/// when the client accepts gzip encoding and the response meets size threshold
///
/// # Arguments
///
/// * `config` - Optional compression configuration. Uses defaults if None.
///
/// # Returns
///
/// A middleware function that compresses HTTP responses
pub(crate) fn compression(
    config: Option<CompressionConfig>,
) -> impl Fn(HttpRequest, HttpResponse) -> FutMiddleware + Send + Sync + 'static {
    let config = config.unwrap_or_default();
    move |req: HttpRequest, mut res| {
        let config = config.clone();
        Box::pin(async move {
            let accepts_gzip = req
                .headers
                .get("Accept-Encoding")
                .map(|v| accepts_gzip_encoding(v))
                .unwrap_or(false);

            if !accepts_gzip {
                return (req, None);
            }
            if res
                .headers
                .get("Content-Encoding")
                .or_else(|| res.headers.get("content-encoding"))
                .is_some()
            {
                return (req, None);
            }
            let body_bytes = match get_response_body_bytes(&res) {
                Some(bytes) => bytes,
                None => return (req, None),
            };

            if body_bytes.len() < config.threshold {
                return (req, None);
            }

            let content_type = res.content_type.as_str();


            if !should_compress_content_type(content_type) {
                return (req, None);
            }

            match compress_data(&body_bytes, config.level) {
                Ok(compressed_body) => {
                    if let Err(_) = set_response_body(&mut res, compressed_body) {
                        return (req, None);
                    }

                    res = res
                        .set_header("Content-Encoding", "gzip")
                        .set_header("Vary", "Accept-Encoding");

                    res.headers.remove("Content-Length");

                    (req, Some(res))
                }
                Err(_) => {
                    (req, None)
                }
            }
        })
    }
}

pub(crate) fn should_compress_content_type(content_type: &str) -> bool {
    let compressible_types = [
        "text/",
        "application/json",
        "application/javascript",
        "application/xml",
        "application/rss+xml",
        "application/atom+xml",
        "application/xhtml+xml",
        "image/svg+xml",
    ];

    let content_type_lower = content_type.to_lowercase();
    compressible_types
        .iter()
        .any(|&ct| content_type_lower.starts_with(ct))
}

/// Compresses data using gzip
pub(crate) fn compress_data(data: &[u8], level: u8) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::new(level.min(9) as u32));
    encoder.write_all(data)?;
    encoder.finish()
}

/// Extracts body bytes from HttpResponse for size checking
pub(crate) fn get_response_body_bytes(response: &HttpResponse) -> Option<Vec<u8>> {
    match &response.body {
        ResponseContentBody::TEXT(text) => Some(text.as_bytes().to_vec()),
        ResponseContentBody::JSON(json) => serde_json::to_vec(json).ok(),
        ResponseContentBody::HTML(html) => Some(html.as_bytes().to_vec()),
        ResponseContentBody::BINARY(bytes) => Some(bytes.to_vec()),
    }
}

/// Sets the compressed body on the HttpResponse
///
/// **Important**: For compressed content, we should always store as BINARY
/// since the compressed data is no longer valid text/JSON/HTML
pub(crate) fn set_response_body(
    response: &mut HttpResponse,
    compressed_body: Vec<u8>,
) -> Result<(), ()> {
    response.body = ResponseContentBody::BINARY(compressed_body.into());
    Ok(())
}

pub(crate) fn accepts_gzip_encoding(header: &str) -> bool {
    header
        .split(',')
        .filter_map(|t| {
            let mut parts = t.trim().split(';');
            let enc = parts.next()?.trim().to_ascii_lowercase();
            let mut q = 1.0_f32;
            for p in parts {
                if let Some(val) = p.trim().strip_prefix("q=") {
                    q = val.parse::<f32>().unwrap_or(0.0);
                }
            }
            Some((enc, q))
        })
        .any(|(enc, q)| q > 0.0 && (enc == "gzip" || enc == "*"))
}
