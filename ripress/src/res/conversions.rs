use bytes::Bytes;
#[cfg(not(feature = "with-wynd"))]
use http_body_util::Full;
use hyper::Response;

#[cfg(not(feature = "with-wynd"))]
use crate::app::api_error::ApiError;
use crate::{
    res::{HttpResponse, ResponseError},
    types::ResponseContentBody,
};

#[cfg(feature = "with-wynd")]
use crate::app::api_error::ApiError;
use crate::helpers::determine_content_type_response;
use crate::{
    res::{response_headers::ResponseHeaders, response_status::StatusCode},
    types::ResponseBodyType,
};
use futures::{stream, StreamExt};
use http_body_util::BodyExt;
#[cfg(feature = "with-wynd")]
use http_body_util::Full;
use hyper::header::{HeaderName, HeaderValue, CONTENT_LENGTH, SET_COOKIE};
use std::convert::Infallible;

impl HttpResponse {
    #[cfg(feature = "with-wynd")]
    #[doc(hidden)]
    pub async fn from_hyper_response(res: &mut Response<Full<Bytes>>) -> Result<Self, ApiError> {
        let collected = res.body_mut().collect().await?;
        let body_bytes = collected.to_bytes();

        let content_type_hdr = res
            .headers()
            .get(hyper::header::CONTENT_TYPE)
            .and_then(|h| h.to_str().ok());

        let content_type = content_type_hdr
            .map(determine_content_type_response)
            .unwrap_or(ResponseBodyType::BINARY);

        let body = match content_type {
            ResponseBodyType::BINARY => ResponseContentBody::new_binary(body_bytes),
            ResponseBodyType::TEXT => {
                let text = String::from_utf8(body_bytes.to_vec())
                    .unwrap_or_else(|_| String::from_utf8_lossy(&body_bytes).into_owned());
                ResponseContentBody::new_text(text)
            }
            ResponseBodyType::JSON => {
                // Avoid panic: if JSON parsing fails, fallback to empty object
                let json_value =
                    serde_json::from_slice(&body_bytes).unwrap_or(serde_json::Value::Null);
                ResponseContentBody::new_json(json_value)
            }
            ResponseBodyType::HTML => {
                let html = String::from_utf8(body_bytes.to_vec())
                    .unwrap_or_else(|_| String::from_utf8_lossy(&body_bytes).into_owned());
                ResponseContentBody::new_html(html)
            }
        };

        // Heuristic for SSE streams: text/event-stream + keep-alive
        let is_event_stream = content_type_hdr
            .map(|ct| ct.eq_ignore_ascii_case("text/event-stream"))
            .unwrap_or(false);
        let is_keep_alive = res
            .headers()
            .get(hyper::header::CONNECTION)
            .and_then(|h| h.to_str().ok())
            .map(|v| v.eq_ignore_ascii_case("keep-alive"))
            .unwrap_or(false);
        let is_stream = is_event_stream && is_keep_alive;

        let status_code = StatusCode::from_u16(res.status().as_u16());
        let mut headers = ResponseHeaders::new();

        for (key, value) in res.headers().iter() {
            if let Ok(v) = value.to_str() {
                headers.insert(key.as_str(), v);
            }
        }
        for value in res.headers().get_all(SET_COOKIE).iter() {
            if let Ok(v) = value.to_str() {
                headers.insert("Set-Cookie", v);
            }
        }

        Ok(HttpResponse {
            body,
            content_type,
            status_code,
            headers,
            cookies: Vec::new(),
            remove_cookies: Vec::new(),
            is_stream,
            stream: Box::pin(stream::empty::<Result<Bytes, ResponseError>>()),
        })
    }
    #[cfg(not(feature = "with-wynd"))]
    #[doc(hidden)]
    pub async fn from_hyper_response(res: &mut Response<Full<Bytes>>) -> Result<Self, ApiError> {
        let collected = res.body_mut().collect().await?;
        let body_bytes = collected.to_bytes();

        // Extract what we need BEFORE taking the HeaderMap
        let content_type_hdr = res
            .headers()
            .get(hyper::header::CONTENT_TYPE)
            .and_then(|h| h.to_str().ok());

        let content_type = content_type_hdr
            .map(determine_content_type_response)
            .unwrap_or(ResponseBodyType::BINARY);

        let body = match content_type {
            ResponseBodyType::BINARY => ResponseContentBody::new_binary(body_bytes),
            ResponseBodyType::TEXT => {
                let text = String::from_utf8(body_bytes.to_vec())
                    .unwrap_or_else(|_| String::from_utf8_lossy(&body_bytes).into_owned());
                ResponseContentBody::new_text(text)
            }
            ResponseBodyType::JSON => {
                let json_value =
                    serde_json::from_slice(&body_bytes).unwrap_or(serde_json::Value::Null);
                ResponseContentBody::new_json(json_value)
            }
            ResponseBodyType::HTML => {
                let html = String::from_utf8(body_bytes.to_vec())
                    .unwrap_or_else(|_| String::from_utf8_lossy(&body_bytes).into_owned());
                ResponseContentBody::new_html(html)
            }
        };

        // Check for SSE stream
        let is_event_stream = content_type_hdr
            .map(|ct| ct.eq_ignore_ascii_case("text/event-stream"))
            .unwrap_or(false);

        let is_keep_alive = res
            .headers()
            .get(hyper::header::CONNECTION)
            .and_then(|h| h.to_str().ok())
            .map(|v| v.eq_ignore_ascii_case("keep-alive"))
            .unwrap_or(false);

        let is_stream = is_event_stream && is_keep_alive;

        let status_code = StatusCode::from_u16(res.status().as_u16());

        // OPTIMIZATION: Take the HeaderMap directly - ZERO parsing!
        let headers = ResponseHeaders::from(std::mem::take(res.headers_mut()));

        Ok(HttpResponse {
            body,
            content_type,
            status_code,
            headers,
            cookies: Vec::new(),
            remove_cookies: Vec::new(),
            is_stream,
            stream: Box::pin(stream::empty::<Result<Bytes, ResponseError>>()),
        })
    }

    #[doc(hidden)]
    pub async fn to_hyper_response(self) -> Result<Response<Full<Bytes>>, Infallible> {
        let body = self.body;

        if self.is_stream {
            let response = Response::builder()
                .status(self.status_code.as_u16())
                .header("Content-Type", "text/event-stream")
                .header("Connection", "keep-alive");

            // OPTIMIZATION: Take the HeaderMap directly instead of iterating
            let mut header_map = self.headers.into_header_map();

            // Add cookies to the header map
            for c in self.cookies.iter() {
                let mut cookie_builder = cookie::Cookie::build((c.name, c.value))
                    .http_only(c.options.http_only)
                    .same_site(match c.options.same_site {
                        crate::res::CookieSameSiteOptions::Lax => cookie::SameSite::Lax,
                        crate::res::CookieSameSiteOptions::Strict => cookie::SameSite::Strict,
                        crate::res::CookieSameSiteOptions::None => cookie::SameSite::None,
                    })
                    .secure(c.options.secure)
                    .path(c.options.path.as_deref().unwrap_or("/"));

                if let Some(domain) = c.options.domain.as_deref() {
                    cookie_builder = cookie_builder.domain(domain);
                }
                if let Some(max_age_secs) = c.options.max_age {
                    cookie_builder =
                        cookie_builder.max_age(cookie::time::Duration::seconds(max_age_secs));
                }
                if let Some(expires_unix) = c.options.expires {
                    if let Ok(odt) = cookie::time::OffsetDateTime::from_unix_timestamp(expires_unix)
                    {
                        cookie_builder = cookie_builder.expires(odt);
                    }
                }

                if let Ok(cookie_value) =
                    HeaderValue::from_bytes(cookie_builder.to_string().as_bytes())
                {
                    header_map.append(SET_COOKIE, cookie_value);
                }
            }

            // Remove cookies
            for cookie_name in self.remove_cookies {
                let expired_cookie = cookie::Cookie::build((cookie_name, ""))
                    .path("/")
                    .max_age(cookie::time::Duration::seconds(0));

                if let Ok(cookie_value) =
                    HeaderValue::from_bytes(expired_cookie.to_string().as_bytes())
                {
                    header_map.append(SET_COOKIE, cookie_value);
                }
            }

            // Collect the stream into a single Bytes value (async)
            let collected_results: Vec<Result<Bytes, ResponseError>> = self.stream.collect().await;

            let bytes = collected_results
                .into_iter()
                .collect::<Result<Vec<Bytes>, _>>()
                .map(|chunks| chunks.concat().into())
                .unwrap_or_else(|_| Bytes::new());

            let mut hyper_response = response.body(Full::from(bytes)).unwrap();

            // Merge our header map into the response
            hyper_response.headers_mut().extend(header_map);

            // Remove Content-Length and set transfer-encoding for streaming
            hyper_response.headers_mut().remove(CONTENT_LENGTH);
            let header_value = HeaderValue::from_static("chunked");
            hyper_response
                .headers_mut()
                .insert(HeaderName::from_static("transfer-encoding"), header_value);

            return Ok(hyper_response);
        } else {
            // Build the base response with content-type
            let mut response = match body {
                ResponseContentBody::JSON(json) => {
                    let json_bytes = serde_json::to_vec(&json).unwrap_or_else(|e| {
                        println!("JSON serialization error: {:?}", e);
                        Vec::from(b"{}")
                    });

                    Response::builder()
                        .status(self.status_code.as_u16())
                        .header("Content-Type", self.content_type.as_str())
                        .body(Full::from(Bytes::from(json_bytes)))
                }
                ResponseContentBody::TEXT(text) => Response::builder()
                    .status(self.status_code.as_u16())
                    .header("Content-Type", self.content_type.as_str())
                    .body(Full::from(Bytes::from(text))),
                ResponseContentBody::HTML(html) => Response::builder()
                    .status(self.status_code.as_u16())
                    .header("Content-Type", self.content_type.as_str())
                    .body(Full::from(Bytes::from(html))),
                ResponseContentBody::BINARY(bytes) => Response::builder()
                    .status(self.status_code.as_u16())
                    .header("Content-Type", self.content_type.as_str())
                    .body(Full::from(Bytes::from(bytes))),
            }
            .unwrap();

            // OPTIMIZATION: Take the HeaderMap directly and merge it
            let mut header_map = self.headers.into_header_map();

            // Remove content-type from our headers if it exists (already set above)
            header_map.remove(hyper::header::CONTENT_TYPE);

            // Add cookies to the header map
            for c in self.cookies {
                let mut cookie_builder = cookie::Cookie::build((c.name, c.value))
                    .http_only(c.options.http_only)
                    .same_site(match c.options.same_site {
                        crate::res::CookieSameSiteOptions::Lax => cookie::SameSite::Lax,
                        crate::res::CookieSameSiteOptions::Strict => cookie::SameSite::Strict,
                        crate::res::CookieSameSiteOptions::None => cookie::SameSite::None,
                    })
                    .secure(c.options.secure)
                    .path(c.options.path.as_deref().unwrap_or("/"));

                if let Some(domain) = c.options.domain.as_deref() {
                    cookie_builder = cookie_builder.domain(domain);
                }
                if let Some(max_age_secs) = c.options.max_age {
                    cookie_builder =
                        cookie_builder.max_age(cookie::time::Duration::seconds(max_age_secs));
                }
                if let Some(expires_unix) = c.options.expires {
                    if let Ok(odt) = cookie::time::OffsetDateTime::from_unix_timestamp(expires_unix)
                    {
                        cookie_builder = cookie_builder.expires(odt);
                    }
                }

                if let Ok(cookie_value) =
                    HeaderValue::from_bytes(cookie_builder.to_string().as_bytes())
                {
                    header_map.append(SET_COOKIE, cookie_value);
                }
            }

            // Remove cookies
            // Remove cookies by sending expired Set-Cookie headers
            for cookie_name in self.remove_cookies {
                let expired_cookie = cookie::Cookie::build((cookie_name, ""))
                    .path("/")
                    .max_age(cookie::time::Duration::seconds(0));

                if let Ok(cookie_value) =
                    HeaderValue::from_bytes(expired_cookie.to_string().as_bytes())
                {
                    header_map.append(SET_COOKIE, cookie_value);
                }
            }
            // Merge all headers at once
            response.headers_mut().extend(header_map);

            return Ok(response);
        }
    }
}
