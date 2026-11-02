use axum::{extract::Request, middleware::Next, response::Response};
pub use gity_observability::{TelemetryGuard, init};
pub async fn request(
  axum::extract::State(state): axum::extract::State<crate::AppState>,
  request: Request,
  next: Next,
) -> Response {
  if request.uri().path() != "/mcp"
    && crate::auth::bearer(request.headers()).is_some_and(|token| token.starts_with("gity_oauth_"))
  {
    use axum::response::IntoResponse;
    return gity_core::AppError::unauthorized("credential audience does not match this API")
      .into_response();
  }
  let actor = crate::auth::principal(&state, request.headers())
    .await
    .ok()
    .map(|principal| principal.user_id);
  let language = gity_core::locale::negotiate_language(
    request
      .headers()
      .get(axum::http::header::ACCEPT_LANGUAGE)
      .and_then(|value| value.to_str().ok())
      .unwrap_or("en"),
  );
  let response = gity_core::locale::with_language(
    language,
    gity_core::audit_context(actor, gity_observability::instrument(request, next)),
  )
  .await;
  gity_core::locale::localize_response(language, response)
}
