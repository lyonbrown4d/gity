use super::*;
use axum::{
    body::Body,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use sha2::{Digest, Sha256};
fn header_text<'a>(h: &'a HeaderMap, key: &str) -> &'a str {
    h.get(key).and_then(|v| v.to_str().ok()).unwrap_or("")
}
fn date(v: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc2822(v)
        .ok()
        .map(|v| v.to_utc())
        .or_else(|| {
            chrono::DateTime::parse_from_str(v, "%a, %d %b %Y %H:%M:%S GMT")
                .ok()
                .map(|v| v.to_utc())
        })
}
fn etag_matches(input: &str, etag: &str, weak: bool) -> bool {
    input.split(',').any(|v| {
        let v = v.trim();
        v == "*" || (if weak { v.trim_start_matches("W/") } else { v }) == etag
    })
}
pub(super) async fn response(
    s: &AppState,
    h: &HeaderMap,
    row: Value,
) -> Result<Response, AppError> {
    let size = num(&row, "byte_size").max(0) as u64;
    let fingerprint = format!(
        "{}:{}:{}:{}",
        num(&row, "id"),
        text(&row, "storage_key"),
        size,
        text(&row, "updated_at")
    );
    let etag = format!(
        "\"{}\"",
        hex::encode(Sha256::digest(fingerprint.as_bytes()))
    );
    let modified = timestamp(text(&row, "updated_at"))
        .ok_or_else(|| AppError::internal("invalid package modification timestamp"))?;
    let modified = chrono::DateTime::from_timestamp(modified.timestamp(), 0).unwrap();
    let mut status = StatusCode::OK;
    let if_match = header_text(h, "if-match");
    if !if_match.is_empty() && !etag_matches(if_match, &etag, false) {
        status = StatusCode::PRECONDITION_FAILED;
    }
    if if_match.is_empty()
        && date(header_text(h, "if-unmodified-since")).is_some_and(|v| modified > v)
    {
        status = StatusCode::PRECONDITION_FAILED;
    }
    let if_none = header_text(h, "if-none-match");
    if status == StatusCode::OK {
        if !if_none.is_empty() {
            if etag_matches(if_none, &etag, true) {
                status = StatusCode::NOT_MODIFIED;
            }
        } else if date(header_text(h, "if-modified-since")).is_some_and(|v| modified <= v) {
            status = StatusCode::NOT_MODIFIED;
        }
    }
    let mut content_range = None;
    let mut length = size;
    let body = if status != StatusCode::OK {
        length = 0;
        Body::empty()
    } else {
        let range = header_text(h, "range");
        let if_range = header_text(h, "if-range");
        let range_allowed = if_range.is_empty()
            || if_range == etag
            || (!if_range.starts_with('"')
                && !if_range.starts_with("W/")
                && date(if_range).is_some_and(|v| modified <= v));
        if !range.is_empty() && range_allowed {
            match bounds(range, size) {
                Some((start, end)) => {
                    status = StatusCode::PARTIAL_CONTENT;
                    length = end - start;
                    content_range = Some(format!("bytes {start}-{}/{size}", end - 1));
                    Body::from(
                        s.storage
                            .get_range(text(&row, "storage_key"), start..end)
                            .await?,
                    )
                }
                None => {
                    status = StatusCode::RANGE_NOT_SATISFIABLE;
                    length = 0;
                    content_range = Some(format!("bytes */{size}"));
                    Body::empty()
                }
            }
        } else {
            Body::from_stream(s.storage.stream(text(&row, "storage_key")).await?)
        }
    };
    let mut out = (status, body).into_response();
    let headers = out.headers_mut();
    headers.insert(header::ETAG, HeaderValue::from_str(&etag).unwrap());
    headers.insert(
        header::LAST_MODIFIED,
        HeaderValue::from_str(&modified.format("%a, %d %b %Y %H:%M:%S GMT").to_string()).unwrap(),
    );
    headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&length.to_string()).unwrap(),
    );
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(text(&row, "content_type"))
            .unwrap_or(HeaderValue::from_static("application/octet-stream")),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment"),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    if let Some(v) = content_range {
        headers.insert(header::CONTENT_RANGE, HeaderValue::from_str(&v).unwrap());
    }
    Ok(out)
}
fn bounds(input: &str, size: u64) -> Option<(u64, u64)> {
    if size == 0 {
        return None;
    }
    let (a, b) = input.strip_prefix("bytes=")?.split_once('-')?;
    if a.is_empty() {
        let count = b.parse::<u64>().ok()?;
        if count == 0 {
            return None;
        }
        return Some((size.saturating_sub(count), size));
    }
    let start = a.parse::<u64>().ok()?;
    let end = if b.is_empty() {
        size
    } else {
        b.parse::<u64>().ok()?.saturating_add(1).min(size)
    };
    if start >= end || start >= size {
        None
    } else {
        Some((start, end))
    }
}
#[cfg(test)]
#[path = "package_binary_tests.rs"]
mod tests;
