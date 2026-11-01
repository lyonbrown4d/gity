use super::*;
use axum::response::IntoResponse;

#[tokio::test]
async fn validation_errors_keep_the_bad_request_status_and_message() {
  let error: gity_core::AppError =
    ArtifactValidationError::new("package version is invalid").into();
  let response = error.into_response();
  assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);

  let body = axum::body::to_bytes(response.into_body(), 1024)
    .await
    .unwrap();
  let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
  assert_eq!(body["message"], "package version is invalid");
}
