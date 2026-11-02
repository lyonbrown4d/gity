use axum::http::{HeaderMap, HeaderValue, header};
use gity_api_contracts::{ApiContext, ApiTransport};

pub(crate) fn context(headers: &HeaderMap, transport: ApiTransport) -> ApiContext {
  let authorization = if let Some(value) = headers.get(header::AUTHORIZATION) {
    Some(value.to_str().unwrap_or("Invalid").to_owned())
  } else {
    crate::registry::token_from_headers(headers).map(|token| format!("Bearer {token}"))
  };
  ApiContext::new(
    authorization,
    headers
      .get(header::ACCEPT_LANGUAGE)
      .and_then(|v| v.to_str().ok())
      .unwrap_or("en")
      .into(),
    uuid::Uuid::now_v7().to_string(),
    transport,
  )
  .with_job_token(
    headers
      .get("job-token")
      .and_then(|v| v.to_str().ok())
      .map(str::to_owned),
  )
}
pub(crate) fn headers(context: &ApiContext) -> HeaderMap {
  let mut headers = HeaderMap::new();
  for (name, value) in [
    ("authorization", context.authorization()),
    ("job-token", context.job_token()),
    ("accept-language", Some(context.language.as_str())),
    ("x-request-id", Some(context.request_id.as_str())),
  ] {
    if let Some(value) = value.and_then(|v| HeaderValue::from_str(v).ok()) {
      headers.insert(name, value);
    }
  }
  headers
}
