use super::super::test_support::fixture;
use super::*;
use std::time::Duration;

async fn git(s: &AppState, args: &[&str]) {
  let output = tokio::process::Command::new(&s.settings.git_bin)
    .arg("--git-dir")
    .arg(s.settings.repo_root.join("20.git"))
    .args(args)
    .output()
    .await
    .unwrap();
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
}

// The shared fixture also supports GITY_TEST_DATABASE_URL for isolated
// PostgreSQL/MySQL runs. Two application services share the database, not a
// process mutex; the transaction lock remains held through completion.
#[tokio::test]
async fn merge_guard_serializes_recovery_abort_and_retry_after_target_rewind() {
  let (_dir, s) = fixture().await;
  let op = super::tests::prepared(&s).await;
  let outcome = s.git.merge_operation("20.git", input(&op)).await.unwrap();
  let MergeOperationOutcome::Applied { commit_sha } = &outcome else {
    panic!("not applied")
  };
  let expected_sha = commit_sha.clone();
  let marker = format!("refs/gity/merge-operations/{}", op.operation_key);
  git(&s, &["update-ref", "-d", &marker]).await;

  let guard = s.collaboration.lock_merge_operation(op.id).await.unwrap();
  let observed = s
    .git
    .merge_operation_outcome("20.git", input(guard.operation()))
    .await
    .unwrap()
    .unwrap();
  let mut second = s.clone();
  second.collaboration = fluxdi::Shared::new(gity_collaboration::CollaborationService::new(
    s.db.connection.clone(),
  ));
  let mut aborted = Box::pin(abort_operation(
    &second,
    op.id,
    "inspected rewind".into(),
    1,
  ));
  assert!(
    tokio::time::timeout(Duration::from_millis(50), &mut aborted)
      .await
      .is_err(),
    "abort escaped the operation transaction before recovery completion"
  );
  git(&s, &["update-ref", "refs/heads/main", &op.target_sha]).await;
  let completed = settle(guard, observed).await.unwrap();
  assert_eq!(completed.status, "applied");
  let aborted = tokio::time::timeout(Duration::from_secs(5), aborted)
    .await
    .unwrap()
    .unwrap();
  assert_eq!(aborted.status, "applied");
  assert_eq!(aborted.result_sha.as_deref(), Some(expected_sha.as_str()));
  // No contradictory immutable rejection was written after the proof became
  // unreachable; stale apply/recovery also trust the serialized DB completion.
  assert!(
    s.git
      .merge_operation_outcome("20.git", input(&op))
      .await
      .unwrap()
      .is_none()
  );
  assert_eq!(apply(&second, op.id).await.unwrap().status, "applied");
  assert_eq!(
    reconcile(&second, op.id).await.unwrap().unwrap().status,
    "applied"
  );
  assert_eq!(
    s.git.resolve_revision("20.git", "main", "").await.unwrap(),
    op.target_sha
  );
  let mr = s.collaboration.merge_request(20, 1).await.unwrap();
  assert_eq!(mr.merge_commit_sha.as_deref(), Some(expected_sha.as_str()));
  let events = s
    .db
    .all(
      "SELECT id FROM project_audit_events WHERE event_name='project.merge_request.merged'",
      vec![],
    )
    .await
    .unwrap();
  assert_eq!(events.len(), 1);
}

#[tokio::test]
async fn merge_guard_abort_wins_before_concurrent_retry_or_recovery() {
  let (_dir, s) = fixture().await;
  let op = super::tests::prepared(&s).await;
  let guard = s.collaboration.lock_merge_operation(op.id).await.unwrap();
  let mut retried = Box::pin(apply(&s, op.id));
  assert!(
    tokio::time::timeout(Duration::from_millis(50), &mut retried)
      .await
      .is_err()
  );
  let outcome = s
    .git
    .abort_merge_operation(
      "20.git",
      input(guard.operation()),
      "cancel before execution".into(),
      1,
    )
    .await
    .unwrap();
  settle(guard, outcome.clone()).await.unwrap();
  assert_eq!(
    tokio::time::timeout(Duration::from_secs(5), retried)
      .await
      .unwrap()
      .unwrap()
      .status,
    "rejected"
  );
  assert_eq!(
    reconcile(&s, op.id).await.unwrap().unwrap().status,
    "rejected"
  );
  assert_eq!(
    s.git
      .merge_operation_outcome("20.git", input(&op))
      .await
      .unwrap(),
    Some(outcome)
  );
  assert_eq!(
    s.git.resolve_revision("20.git", "main", "").await.unwrap(),
    op.target_sha
  );
}

#[tokio::test]
async fn repository_http_cannot_read_or_publish_merge_receipts() {
  use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
  };
  use tower::ServiceExt;
  let (_dir, s) = fixture().await;
  let op = super::tests::prepared(&s).await;
  apply(&s, op.id).await.unwrap();
  let marker = format!("refs/gity/merge-operations/{}", op.operation_key);
  let snapshot = s.git.reference_snapshot("20.git").await.unwrap();
  let receipt_sha = snapshot.refs[&marker].clone();
  let router = crate::repository::routes().with_state(s.clone());
  for revision in [&marker, &receipt_sha] {
    let response = router
      .clone()
      .oneshot(
        Request::builder()
          .uri(format!(
            "/projects/20/repository/blob?ref_name={revision}&path=receipt.json"
          ))
          .header("authorization", "Bearer alice")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = to_bytes(response.into_body(), 4096).await.unwrap();
    assert!(!String::from_utf8_lossy(&body).contains(&op.operation_key));
    let response = router
      .clone()
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/projects/20/repository/branches")
          .header("authorization", "Bearer alice")
          .header("content-type", "application/json")
          .body(Body::from(
            json!({"name":"leaked", "source_ref":revision}).to_string(),
          ))
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
  }
  let response = router
    .oneshot(
      Request::builder()
        .uri(format!(
          "/projects/20/repository/blob?ref_name={}&path=base",
          op.target_sha
        ))
        .header("authorization", "Bearer alice")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);
}
