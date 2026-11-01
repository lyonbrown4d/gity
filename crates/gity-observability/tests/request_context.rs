use axum::{
  Router,
  body::{Body, to_bytes},
  http::{HeaderMap, Request},
  routing::get,
};
use tower::ServiceExt;
#[tokio::test]
async fn request_id_is_bounded_and_shared_by_downstream_and_response() {
  for supplied in [
    None,
    Some("test-request-42".to_owned()),
    Some("x".repeat(100)),
  ] {
    let app = Router::new()
      .route(
        "/probe",
        get(
          |headers: HeaderMap| async move { headers["x-request-id"].to_str().unwrap().to_owned() },
        ),
      )
      .layer(axum::middleware::from_fn(gity_observability::instrument));
    let mut request = Request::builder().uri("/probe");
    if let Some(id) = &supplied {
      request = request.header("x-request-id", id);
    }
    let response = app
      .oneshot(request.body(Body::empty()).unwrap())
      .await
      .unwrap();
    let id = response.headers()["x-request-id"]
      .to_str()
      .unwrap()
      .to_owned();
    if supplied.as_ref().is_some_and(|value| value.len() < 64) {
      assert_eq!(Some(&id), supplied.as_ref());
    } else {
      assert_eq!(uuid::Uuid::parse_str(&id).unwrap().get_version_num(), 7);
    }
    let body = to_bytes(response.into_body(), 1024).await.unwrap();
    assert_eq!(body.as_ref(), id.as_bytes());
  }
}
