#![warn(missing_docs)]
use std::{fmt::Display, future::Future, sync::Arc};

#[cfg(feature = "with-wynd")]
use crate::middlewares::WyndMiddleware;
use crate::req::body::RequestBodyType;
use crate::res::{response_headers::ResponseHeaders, ResponseBodyType};
use crate::{
    app::api_error::ApiError,
    middlewares::Middleware,
    req::{query_params::QueryParams, HttpRequest},
    res::HttpResponse,
    types::RouteHandlerReturnType,
};
use http_body_util::Full;
use hyper::{body::Bytes, Request, Response};
use mime::Mime;
use routerify_ng::RequestInfo;
use url::form_urlencoded::Serializer;

/// Stores pre-middleware response headers that need to be merged into the final response.
///
/// When a pre-middleware modifies the response but returns `None` (to continue processing),
/// the modified headers are stored in the request extensions so they can be merged into
/// the final response from the route handler.
#[derive(Clone, Debug)]
pub(crate) struct PreMiddlewareResponseHeaders {
    pub(crate) headers: ResponseHeaders,
}

pub(crate) async fn exec_pre_middleware(
    mut req: Request<Full<Bytes>>,
    middleware: Arc<Middleware>,
) -> Result<Request<Full<Bytes>>, ApiError> {
    let mw_func = &middleware.func;

    if path_matches(middleware.path.as_str(), req.uri().path()) {
        let our_res = HttpResponse::new();

        let our_req = HttpRequest::from_hyper_request(&mut req)
            .await
            .map_err(ApiError::from)?;

        let (modified_req, maybe_res) = mw_func(our_req, our_res).await;

        match maybe_res {
            None => {
                let hyper_req = modified_req.to_hyper_request()?;
                Ok(hyper_req)
            }
            Some(res) => {
                if modified_req.method == crate::types::HttpMethods::OPTIONS {
                    return Err(ApiError::Generic(res));
                }
                use crate::helpers::PreMiddlewareResponseHeaders;

                let headers = res.headers.clone();
                let mut hyper_req = modified_req.to_hyper_request()?;

                hyper_req
                    .extensions_mut()
                    .insert(PreMiddlewareResponseHeaders { headers });

                Ok(hyper_req)
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
            let hyper_res = res.to_hyper_response().await.unwrap();
            return Ok(hyper_res);
        }
    }
}

#[cfg(feature = "with-wynd")]
pub(crate) async fn exec_wynd_middleware(
    req: Request<Full<Bytes>>,
    middleware: Arc<WyndMiddleware>,
) -> Result<Request<Full<Bytes>>, ApiError> {
    if path_matches(middleware.path.as_str(), req.uri().path()) {
        let mw_func = &middleware.func;
        let response = mw_func(req).await;

        match response {
            Err(_e) => {
                return Err(ApiError::Generic(
                    HttpResponse::new()
                        .internal_server_error()
                        .text("WebSocket handler error"),
                ));
            }
            Ok(res) => {
                if res.status() == hyper::StatusCode::SWITCHING_PROTOCOLS {
                    return Err(ApiError::WebSocketUpgrade(res));
                }

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

pub(crate) fn extract_boundary(content_type: &str) -> Option<String> {
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

    let mut pos = match find_subsequence(body, boundary_start_bytes) {
        Some(p) => p + boundary_start_bytes.len(),
        None => return (Vec::new(), Vec::new()),
    };

    if body.get(pos..pos + 2) == Some(b"\r\n") {
        pos += 2;
    }

    let mut fields: Vec<(&'a str, &'a str)> = Vec::new();
    let mut file_parts: Vec<(Vec<u8>, Option<&'a str>)> = Vec::new();

    loop {
        let header_end_rel = match find_subsequence(&body[pos..], b"\r\n\r\n") {
            Some(i) => i,
            None => return (fields, file_parts),
        };
        let headers_bytes = &body[pos..pos + header_end_rel];
        let headers_str = match std::str::from_utf8(headers_bytes) {
            Ok(s) => s,
            Err(_) => return (fields, file_parts),
        };
        let content_start = pos + header_end_rel + 4;

        let next_boundary_rel = match find_subsequence(&body[content_start..], boundary_next_bytes)
        {
            Some(i) => i,
            None => match find_subsequence(&body[content_start..], boundary_close_bytes) {
                Some(i2) => i2,
                None => return (fields, file_parts),
            },
        };
        let content_end = content_start + next_boundary_rel;

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

        pos = content_end;
        if body.get(pos..pos + boundary_next_bytes.len()) == Some(boundary_next_bytes) {
            pos += boundary_next_bytes.len();
        } else if body.get(pos..pos + boundary_close_bytes.len()) == Some(boundary_close_bytes) {
            return (fields, file_parts);
        } else {
            match find_subsequence(&body[pos..], boundary_next_bytes) {
                Some(rel) => pos += rel + boundary_next_bytes.len(),
                None => return (fields, file_parts),
            }
        }

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

pub(crate) fn box_future<F>(future: F) -> RouteHandlerReturnType
where
    F: Future<Output = HttpResponse> + Send + 'static,
{
    Box::pin(future)
}

/// Trait for extracting a type from an HTTP request reference.
///
/// Types that implement `FromRequest` can be constructed from a borrowed `HttpRequest`.
/// This is commonly used to extract body data, headers, params, and other request-specific info
/// into strongly-typed values for route handlers.
///
/// The associated `Error` type enables fallible extraction.  
///
/// # Example
/// ```
/// use ripress::req::HttpRequest;
/// use ripress::helpers::FromRequest;
///
/// struct MyType;
///
/// impl FromRequest for MyType {
///     type Error = String;
///
///     fn from_request(_req: &HttpRequest) -> Result<Self, Self::Error> {
///         Ok(MyType)
///     }
/// }
/// ```
pub trait FromRequest: Sized {
    /// The type of error returned when extraction fails.
    type Error: Display;

    /// Attempt to extract Self from the given HTTP request reference.
    ///
    /// Returns `Ok(Self)` if extraction is successful, or `Err(Self::Error)` if it fails.
    fn from_request(req: &HttpRequest) -> Result<Self, Self::Error>;
}

/// A helper trait for extracting parameters from an owned `HttpRequest`.
///
/// This trait allows `HttpRequest` to be passed directly without cloning,
/// while other types can still use `FromRequest` for extraction.
///
/// ## Multiple Extractors
///
/// This trait is implemented for tuples of 2, 3, 4, and 5 extractors, allowing
/// you to use multiple extractors in a single route handler:
///
/// ```rust,ignore
/// use ripress::{app::App, req::{body::JsonBody, route_params::Params}};
///
/// // Two extractors
/// app.get("/users/:id", |(body, params): (JsonBody<UserData>, Params<UserId>), res| async move {
///     // ...
/// });
///
/// // Up to 5 extractors are supported
/// app.post("/", |(a, b, c, d, e): (Extractor1, Extractor2, Extractor3, Extractor4, Extractor5), res| async move {
///     // ...
/// });
/// ```
pub trait ExtractFromOwned: Sized {
    /// The associated error type returned when extraction fails.
    type Error: Display;

    /// Extract the parameter from an owned `HttpRequest`.
    ///
    /// For `HttpRequest`, this simply moves the request (no clone).
    /// For other types, this uses `FromRequest` which may clone the extracted type.
    fn extract_from_owned(req: HttpRequest) -> Result<Self, Self::Error>;
}

impl ExtractFromOwned for HttpRequest {
    type Error = std::convert::Infallible;

    fn extract_from_owned(req: HttpRequest) -> Result<Self, Self::Error> {
        Ok(req)
    }
}

impl<T> ExtractFromOwned for T
where
    T: FromRequest,
{
    type Error = <T as FromRequest>::Error;

    fn extract_from_owned(req: HttpRequest) -> Result<Self, Self::Error> {
        T::from_request(&req)
    }
}

/// Macro to generate tuple implementations for ExtractFromOwned up to N.
macro_rules! impl_extract_from_owned_tuples {
    ($($len:literal: ($($T:ident),+)),+) => {
        $(
            impl<$($T),+> ExtractFromOwned for ($($T,)+)
            where
                $($T: ExtractFromOwned + Send + 'static),+
            {
                type Error = String;

                fn extract_from_owned(req: HttpRequest) -> Result<Self, Self::Error> {
                    $(
                        #[allow(non_snake_case)]
                        let $T = {
                            $T::extract_from_owned(req.clone())
                                .map_err(|e| format!(
                                    concat!(
                                        "Failed to extract ",
                                        stringify!($T),
                                        " parameter: {}"
                                    ),
                                    e
                                ))?
                        };
                    )+

                    Ok(($($T,)+))
                }
            }
        )+
    };
}

impl_extract_from_owned_tuples!(
    1: (A),
    2: (A, B),
    3: (A, B, C),
    4: (A, B, C, D),
    5: (A, B, C, D, E),
    6: (A, B, C, D, E, F),
    7: (A, B, C, D, E, F, G),
    8: (A, B, C, D, E, F, G, H),
    9: (A, B, C, D, E, F, G, H, I),
    10: (A, B, C, D, E, F, G, H, I, J),
    11: (A, B, C, D, E, F, G, H, I, J, K),
    12: (A, B, C, D, E, F, G, H, I, J, K, L),
    13: (A, B, C, D, E, F, G, H, I, J, K, L, M),
    14: (A, B, C, D, E, F, G, H, I, J, K, L, M, N),
    15: (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O),
    16: (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P)
);

pub(crate) fn determine_content_type_request(content_type: &str) -> RequestBodyType {
    match content_type.parse::<Mime>() {
        Ok(mime_type) => match (mime_type.type_(), mime_type.subtype()) {
            (mime::APPLICATION, mime::JSON) => RequestBodyType::JSON,
            (mime::APPLICATION, subtype) if subtype == "x-www-form-urlencoded" => {
                RequestBodyType::FORM
            }
            (mime::MULTIPART, subtype) if subtype == "form-data" => RequestBodyType::MultipartForm,
            (mime::TEXT, _) => RequestBodyType::TEXT,
            (mime::APPLICATION, subtype) if subtype.as_str().ends_with("+json") => {
                RequestBodyType::JSON
            }
            (mime::APPLICATION, subtype)
                if subtype == "xml" || subtype.as_str().ends_with("+xml") =>
            {
                RequestBodyType::TEXT
            }
            _ => RequestBodyType::BINARY,
        },
        Err(_) => RequestBodyType::BINARY,
    }
}

pub(crate) fn determine_content_type_response(content_type: &str) -> ResponseBodyType {
    match content_type.parse::<Mime>() {
        Ok(mime_type) => match (mime_type.type_(), mime_type.subtype()) {
            (mime::APPLICATION, mime::JSON) => ResponseBodyType::JSON,
            (mime::TEXT, subtype) => {
                if subtype == "html" {
                    ResponseBodyType::HTML
                } else {
                    ResponseBodyType::TEXT
                }
            }
            (mime::APPLICATION, subtype)
                if subtype.as_str().ends_with("+json") || subtype == "vnd.api" =>
            {
                ResponseBodyType::JSON
            }
            (mime::APPLICATION, subtype)
                if subtype == "xml" || subtype.as_str().ends_with("+xml") =>
            {
                ResponseBodyType::TEXT
            }
            _ => ResponseBodyType::BINARY,
        },
        Err(_) => ResponseBodyType::BINARY,
    }
}
