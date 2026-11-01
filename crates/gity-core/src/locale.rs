use std::future::Future;
tokio::task_local! {static LANGUAGE:&'static str;}

pub fn current_language() -> &'static str {
  LANGUAGE.try_with(|value| *value).unwrap_or("en")
}
pub async fn with_language<F: Future>(language: &str, future: F) -> F::Output {
  LANGUAGE
    .scope(gity_i18n::normalize_locale(language), future)
    .await
}

/// Select the highest quality supported language; preserve header order on ties.
pub fn negotiate_language(header: &str) -> &'static str {
  let mut best = ("en", 0.0f32);
  for entry in header.split(',') {
    let mut fields = entry.trim().split(';');
    let tag = fields.next().unwrap_or_default().trim();
    let base = tag.split(['-', '_']).next().unwrap_or_default();
    if !base.eq_ignore_ascii_case("zh") && !base.eq_ignore_ascii_case("en") && base != "*" {
      continue;
    }
    let quality = fields
      .find_map(|v| v.trim().strip_prefix("q="))
      .map_or(Some(1.0), |v| v.parse::<f32>().ok());
    if let Some(q) = quality.filter(|v| v.is_finite() && *v > 0.0 && *v <= 1.0)
      && q > best.1
    {
      best = (gity_i18n::normalize_locale(tag), q);
    }
  }
  best.0
}
#[cfg(test)]
mod tests;

/// Request-local localization shared by API and independently deployed registries.
pub async fn request(
  request: axum::extract::Request,
  next: axum::middleware::Next,
) -> axum::response::Response {
  let language = negotiate_language(
    request
      .headers()
      .get(axum::http::header::ACCEPT_LANGUAGE)
      .and_then(|value| value.to_str().ok())
      .unwrap_or("en"),
  );
  localize_response(language, with_language(language, next.run(request)).await)
}
pub fn localize_response(
  language: &'static str,
  mut response: axum::response::Response,
) -> axum::response::Response {
  response.headers_mut().insert(
    axum::http::header::CONTENT_LANGUAGE,
    axum::http::HeaderValue::from_static(language),
  );
  response.headers_mut().append(
    axum::http::header::VARY,
    axum::http::HeaderValue::from_static("Accept-Language"),
  );
  response
}
