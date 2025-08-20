use crate::{
    app::{Middleware, api_error::ApiError},
    req::{HttpRequest, query_params::QueryParams},
    res::HttpResponse,
};
use hyper::{Body, Request};
use url::form_urlencoded::Serializer;

pub(crate) async fn exec_middleware(
    mut req: Request<Body>,
    middleware: Middleware,
) -> Result<Request<Body>, ApiError> {
    let mw_func = middleware.func;

    let our_res = HttpResponse::new();
    let our_req = HttpRequest::from_hyper_request(&mut req)
        .await
        .map_err(ApiError::from)?;

    if path_matches(middleware.path.as_str(), our_req.path.as_str()) {
        let (modified_req, maybe_res) = mw_func(our_req, our_res).await;

        match maybe_res {
            None => {
                return modified_req.to_hyper_request().map_err(ApiError::from);
            }
            Some(res) => {
                return Err(ApiError::Generic(res));
            }
        }
    } else {
        our_req.to_hyper_request().map_err(ApiError::from)
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
pub(crate) fn parse_multipart_form(
    body: &[u8],
    boundary: &str,
) -> (Vec<(String, String)>, Vec<(Vec<u8>, Option<String>)>) {
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

    let mut fields: Vec<(String, String)> = Vec::new();
    let mut file_parts: Vec<(Vec<u8>, Option<String>)> = Vec::new();

    loop {
        // Find end of headers (CRLFCRLF)
        let header_end_rel = match find_subsequence(&body[pos..], b"\r\n\r\n") {
            Some(i) => i,
            None => return (fields, file_parts),
        };
        let headers_bytes = &body[pos..pos + header_end_rel];
        let headers_str = String::from_utf8_lossy(headers_bytes);
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
        let mut field_name: Option<String> = None;
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
                    match key.as_str() {
                        "name" if !val.is_empty() => {
                            field_name = Some(val.to_string());
                        }
                        "filename" | "filename*" if !val.is_empty() => {
                            is_file_part = true;
                        }
                        _ => {}
                    }
                }
            }
        }

        if is_file_part {
            // Collect ALL file parts, not just the first one
            let file_bytes = trim_trailing_crlf(&body[content_start..content_end]).to_vec();
            // Pass the field_name instead of original_filename for the mapping
            file_parts.push((file_bytes, field_name));
        } else if let Some(name) = field_name {
            let value_bytes = trim_trailing_crlf(&body[content_start..content_end]);
            let value = String::from_utf8_lossy(value_bytes).to_string();
            fields.push((name, value));
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
