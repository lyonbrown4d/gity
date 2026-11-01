use axum::http::{HeaderMap, HeaderName, StatusCode};

pub(super) fn is_hop_header(name: &HeaderName, headers: &HeaderMap) -> bool {
  const FIXED: [&str; 8] = [
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
  ];
  if FIXED.contains(&name.as_str()) {
    return true;
  }
  headers
    .get(axum::http::header::CONNECTION)
    .and_then(|value| value.to_str().ok())
    .is_some_and(|tokens| {
      tokens
        .split(',')
        .any(|token| token.trim().eq_ignore_ascii_case(name.as_str()))
    })
}

pub(super) fn is_identity_header(name: &HeaderName) -> bool {
  matches!(
    name.as_str(),
    "x-gity-user-id"
      | "x-gity-project-id"
      | "x-authenticated-user"
      | "x-remote-user"
      | "x-forwarded-user"
      | "x-forwarded-groups"
  )
}

pub(super) fn validate_redirect(
  location: &str,
  upstream: &str,
) -> Result<Option<String>, (StatusCode, &'static str)> {
  let base = reqwest::Url::parse(upstream)
    .map_err(|_| (StatusCode::BAD_GATEWAY, "invalid registry upstream"))?;
  let parsed = base
    .join(location)
    .map_err(|_| (StatusCode::BAD_GATEWAY, "invalid registry redirect"))?;
  if parsed.origin() != base.origin() {
    return Err((
      StatusCode::BAD_GATEWAY,
      "registry redirect is not same-origin",
    ));
  }
  Ok(Some(format!(
    "{}{}",
    parsed.path(),
    parsed
      .query()
      .map(|query| format!("?{query}"))
      .unwrap_or_default()
  )))
}

pub(super) fn map_storage_location(
  location: &str,
  storage_repository: &str,
  public_repository: &str,
) -> Result<String, (StatusCode, &'static str)> {
  if storage_repository.is_empty() || public_repository.is_empty() {
    return Ok(location.to_owned());
  }
  let prefix = format!("/v2/{storage_repository}");
  let suffix = location
    .strip_prefix(&prefix)
    .filter(|suffix| suffix.is_empty() || suffix.starts_with('/') || suffix.starts_with('?'))
    .ok_or((StatusCode::BAD_GATEWAY, "invalid registry upload location"))?;
  Ok(format!("/v2/{public_repository}{suffix}"))
}

pub(super) fn validate_auth_realm(
  challenge: &str,
  allowed_hosts: &std::collections::BTreeSet<String>,
) -> Result<(), (StatusCode, &'static str)> {
  if let Some(realm) = challenge
    .split("realm=\"")
    .nth(1)
    .and_then(|part| part.split('"').next())
  {
    let url = reqwest::Url::parse(realm)
      .map_err(|_| (StatusCode::BAD_GATEWAY, "invalid registry token realm"))?;
    let host = url
      .host_str()
      .ok_or((StatusCode::BAD_GATEWAY, "invalid registry token realm"))?;
    if !allowed_hosts.contains(host) {
      return Err((
        StatusCode::BAD_GATEWAY,
        "registry token realm host is not allowed",
      ));
    }
  }
  Ok(())
}
