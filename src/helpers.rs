#![warn(missing_docs)]
use std::sync::Arc;

#[cfg(feature = "with-wynd")]
use crate::middlewares::WyndMiddleware;
use crate::{
    app::api_error::ApiError,
    middlewares::Middleware,
    req::{HttpRequest, query_params::QueryParams},
    res::HttpResponse,
    types::{Fut, FutMiddleware},
};
use http_body_util::Full;
#[cfg(feature = "with-wynd")]
use hyper::body::Body;
use hyper::{Request, Response, body::Bytes};
use routerify_ng::RequestInfo;
use url::form_urlencoded::Serializer;

pub(crate) async fn exec_pre_middleware(
    mut req: Request<Full<Bytes>>,
    middleware: Arc<Middleware>,
) -> Result<Request<Full<Bytes>>, ApiError> {
    let mw_func = &middleware.func;

    if path_matches(middleware.path.as_str(), req.uri().path()) {
        let our_res = HttpResponse::new();

        // Work with the original Incoming request directly
        let our_req = HttpRequest::from_hyper_request(&mut req)
            .await
            .map_err(ApiError::from)?;

        let (modified_req, maybe_res) = mw_func(our_req, our_res).await;

        match maybe_res {
            None => {
                let hyper_req = modified_req.to_hyper_request()?;
                return Ok(hyper_req);
            }
            Some(res) => {
                return Err(ApiError::Generic(res));
            }
        }
    } else {
        Ok(req)
    }
}

pub(crate) async fn exec_post_middleware(
    mut res: Response<Full<Bytes>>,
    middleware: Arc<Middleware>,
    info: RequestInfo,
) -> Result<Response<Full<Bytes>>, ApiError> {
    let mw_func = &middleware.func;

    let mut our_req = HttpRequest::from_request_info(&info);

    if let Some(data) = info.data::<routerify_ng::RouteParams>() {
        data.iter().for_each(|(key, value)| {
            our_req.set_param(key, value);
        });
    }

    let our_res = match HttpResponse::from_hyper_response(&mut res).await {
        Ok(res) => res,
        Err(e) => {
            return Err(ApiError::Generic(
                HttpResponse::new()
                    .internal_server_error()
                    .text(e.to_string()),
            ));
        }
    };

    let (_, maybe_res) = mw_func(our_req, our_res).await;
    match maybe_res {
        None => Ok(res),
        Some(res) => {
            // Infallible means this can never fail, so unwrap is safe
            let hyper_res = res.to_hyper_response().await.unwrap();
            return Ok(hyper_res);
        }
    }
}

#[cfg(feature = "with-wynd")]
pub(crate) async fn exec_wynd_middleware(
    req: Request<Full<Bytes>>,
    middleware: WyndMiddleware,
) -> Result<Request<Full<Bytes>>, ApiError> {
    if path_matches(middleware.path.as_str(), req.uri().path()) {
        let mw_func = middleware.func;
        let response = mw_func(req).await;

        match response {
            Err(_e) => {
                // If the handler returns an error, we can't continue with the original request
                // since it may have been consumed. Return an error response.
                return Err(ApiError::Generic(
                    HttpResponse::new()
                        .internal_server_error()
                        .text("WebSocket handler error"),
                ));
            }
            Ok(res) => {
                // Check if this is a WebSocket upgrade response (status 101)
                if res.status() == hyper::StatusCode::SWITCHING_PROTOCOLS {
                    // For WebSocket upgrades, we need to return the response directly
                    // WITHOUT converting it, to preserve hyper's upgrade mechanism
                    return Err(ApiError::WebSocketUpgrade(res));
                }

                // For normal responses, do the conversion as before
                let mut res = res;
                return Err(ApiError::Generic(
                    HttpResponse::from_hyper_response(&mut res).await?,
                ));
            }
        }
    } else {
        Ok(req)
    }
}

pub(crate) fn path_matches(prefix: &str, path: &str) -> bool {
    let is_slash = prefix == "/" || prefix.ends_with('/');
    if is_slash {
        path == prefix
            || path.starts_with(&(prefix.to_string()))
            || path.starts_with(&(prefix.to_string() + "/"))
    } else {
        path == prefix || path.starts_with(&(prefix.to_string() + "/"))
    }
}

pub(crate) fn get_all_query(queries: &QueryParams) -> String {
    let mut ser = Serializer::new(String::new());
    for (k, v) in queries.iter() {
        ser.append_pair(k, v);
    }
    ser.finish()
}

// Helper functions that need to be added to the file:

pub(crate) fn extract_boundary(content_type: &str) -> Option<String> {
    // Prefer robust parsing via the `mime` crate; handles quoting and spacing.
    if let Ok(m) = content_type.parse::<mime::Mime>() {
        if m.type_() == mime::MULTIPART {
            if let Some(b) = m.get_param("boundary") {
                let s = b.as_str();
                if !s.is_empty() {
                    return Some(s.to_string());
                }
            }
        }
    }

    // Fallback: best-effort manual parse for non-standard content types
    for part in content_type.split(';').map(|s| s.trim()) {
        let (k, v) = match part.split_once('=') {
            Some((k, v)) => (k.trim(), v.trim()),
            None => continue,
        };
        if k.eq_ignore_ascii_case("boundary") {
            let b = v.trim_matches('"');
            if !b.is_empty() {
                return Some(b.to_string());
            }
        }
    }

    None
}

pub(crate) fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

// Updated multipart parser that extracts text fields and ALL file parts
// Returns (fields, file_parts) where file_parts is Vec<(bytes, field_name)>
pub(crate) fn parse_multipart_form<'a>(
    body: &'a [u8],
    boundary: &String,
) -> (Vec<(&'a str, &'a str)>, Vec<(Vec<u8>, Option<&'a str>)>) {
    let boundary_start = format!("--{}", boundary);
    let boundary_start_bytes = boundary_start.as_bytes();
    let boundary_next = format!("\r\n--{}", boundary);
    let boundary_next_bytes = boundary_next.as_bytes();
    let boundary_close = format!("--{}--", boundary);
    let boundary_close_bytes = boundary_close.as_bytes();

    // Find the first boundary
    let mut pos = match find_subsequence(body, boundary_start_bytes) {
        Some(p) => p + boundary_start_bytes.len(),
        None => return (Vec::new(), Vec::new()),
    };

    // Skip optional CRLF after the first boundary
    if body.get(pos..pos + 2) == Some(b"\r\n") {
        pos += 2;
    }

    let mut fields: Vec<(&'a str, &'a str)> = Vec::new();
    let mut file_parts: Vec<(Vec<u8>, Option<&'a str>)> = Vec::new();

    loop {
        // Find end of headers (CRLFCRLF)
        let header_end_rel = match find_subsequence(&body[pos..], b"\r\n\r\n") {
            Some(i) => i,
            None => return (fields, file_parts),
        };
        let headers_bytes = &body[pos..pos + header_end_rel];
        // SAFETY: headers_bytes is ASCII and safe as UTF-8
        let headers_str = match std::str::from_utf8(headers_bytes) {
            Ok(s) => s,
            Err(_) => return (fields, file_parts),
        };
        let content_start = pos + header_end_rel + 4;

        // Locate the next boundary (start of next part or closing)
        let next_boundary_rel = match find_subsequence(&body[content_start..], boundary_next_bytes)
        {
            Some(i) => i,
            None => {
                // Try close boundary without preceding CRLF (edge case)
                match find_subsequence(&body[content_start..], boundary_close_bytes) {
                    Some(i2) => i2,
                    None => return (fields, file_parts),
                }
            }
        };
        let content_end = content_start + next_boundary_rel;

        // Parse Content-Disposition to determine field name and if this is a file part
        let mut is_file_part = false;
        let mut field_name: Option<&'a str> = None;
        for line in headers_str.lines() {
            let l = line.trim();
            if l.to_ascii_lowercase().starts_with("content-disposition:") {
                let after_colon = l.splitn(2, ':').nth(1).unwrap_or("").trim();
                for param in after_colon.split(';') {
                    let param = param.trim();
                    let (k, v) = match param.split_once('=') {
                        Some((k, v)) => (k.trim(), v.trim()),
                        None => continue,
                    };
                    let key = k.to_ascii_lowercase();
                    let val = extract_quoted_or_token(v);

                    // get &str out of v slice using header_str lifetime
                    let v_offset = v.as_ptr() as usize - headers_str.as_ptr() as usize;
                    let val_offset = if let Some(start) = v.find('"') {
                        let val2 = &v[start + 1..];
                        if let Some(_) = val2.find('"') {
                            v_offset + start + 1
                        } else {
                            v_offset
                        }
                    } else {
                        v_offset
                    };
                    let val_len = val.len();

                    let val_str: &'a str =
                        if val_len > 0 && val_offset + val_len <= headers_str.len() {
                            // SAFETY: valid substring
                            unsafe {
                                std::str::from_utf8_unchecked(
                                    &headers_str.as_bytes()[val_offset..val_offset + val_len],
                                )
                            }
                        } else {
                            continue;
                        };

                    match key.as_str() {
                        "name" if !val_str.is_empty() => field_name = Some(val_str),
                        "filename" | "filename*" if !val_str.is_empty() => {
                            is_file_part = true;
                        }
                        _ => {}
                    }
                }
            }
        }

        if is_file_part {
            let file_bytes = trim_trailing_crlf(&body[content_start..content_end]).to_vec();
            file_parts.push((file_bytes, field_name));
        } else if let Some(name) = field_name {
            let value_bytes = trim_trailing_crlf(&body[content_start..content_end]);
            if let Ok(value_str) = std::str::from_utf8(value_bytes) {
                fields.push((name, value_str));
            }
        }

        // Move to the next part
        pos = content_end;
        // Step to the boundary marker and beyond
        if body.get(pos..pos + boundary_next_bytes.len()) == Some(boundary_next_bytes) {
            pos += boundary_next_bytes.len();
        } else if body.get(pos..pos + boundary_close_bytes.len()) == Some(boundary_close_bytes) {
            // End reached
            return (fields, file_parts);
        } else {
            // Try to realign to the next boundary start
            match find_subsequence(&body[pos..], boundary_next_bytes) {
                Some(rel) => pos += rel + boundary_next_bytes.len(),
                None => return (fields, file_parts),
            }
        }

        // Skip CRLF after boundary if present
        if body.get(pos..pos + 2) == Some(b"\r\n") {
            pos += 2;
        }
    }
}

pub(crate) fn trim_trailing_crlf(slice: &[u8]) -> &[u8] {
    if slice.ends_with(b"\r\n") {
        &slice[..slice.len() - 2]
    } else {
        slice
    }
}

pub(crate) fn extract_quoted_or_token(input: &str) -> &str {
    let val = input.trim();
    if let Some(start) = val.find('"') {
        let val2 = &val[start + 1..];
        if let Some(end) = val2.find('"') {
            &val2[..end]
        } else {
            val
        }
    } else {
        val.split(';').next().unwrap_or(val).trim()
    }
}

pub(crate) fn box_future<F>(future: F) -> Fut
where
    F: Future<Output = HttpResponse> + Send + 'static,
{
    Box::pin(future)
}

pub(crate) fn box_future_middleware<F>(future: F) -> FutMiddleware
where
    F: Future<Output = (HttpRequest, Option<HttpResponse>)> + Send + 'static,
{
    Box::pin(future)
}

/// A macro for convenient construction of middleware vectors (`Middlewares`).
///
/// # Usage
///
/// The `middlewares!` macro simplifies the creation of middleware lists by allowing you to specify
/// route patterns and corresponding middleware closures in a concise and readable way.
///
/// Each element is a tuple in the form: `("/path", |req, res| { ... })`.
///
/// # Example
///
/// ```rust
/// use ripress::{app::App, types::Middlewares, middlewares};
///
/// let pre_middlewares: Middlewares = middlewares![
///     ("/", |req, _res| Box::pin(async move { (req, None) })),
///     ("/admin", |req, _res| Box::pin(async move { (req, None) })),
/// ];
/// ```
///
/// # Output
///
/// Expands into a `Vec<(&'static str, Box<dyn Fn(...) -> ...>)>` ready for
/// use with `App::use_pre_middlewares()` or `App::use_post_middlewares()`.
#[macro_export]
macro_rules! middlewares {
    ( $( ($path:expr, $handler:expr) ),* $(,)? ) => {
        {
            let mut vec: $crate::types::Middlewares = Vec::new();
            $(
                vec.push((
                    $path,
                    Box::new($handler)
                ));
            )*
            vec
        }
    };
}
