use super::test_support::{fixture, request};
use super::*;
use axum::http::StatusCode;

#[tokio::test]
async fn approval_rule_writes_preserve_audit_actor_scope_and_rollback() {
  let (_dir, s) = fixture().await;
  let (status, created) = request(
    &s,
    "POST",
    "/projects/20/merge-request-approval-rules",
    "alice",
    json!({"name":"Review","approvals_required":1,"eligible_user_ids":[2]}),
  )
  .await;
  assert_eq!(status, StatusCode::OK);
  let id = created["data"]["id"].as_str().unwrap();
  let snapshots = s.db.all(
    "SELECT a.operation,r.actor FROM project_merge_request_approval_rule_audits a JOIN audit_revisions r ON r.id=a.revision_id WHERE a.project_merge_request_approval_rule_id=?",
    vec![id.parse::<i64>().unwrap().into()],
  ).await.unwrap();
  assert_eq!(snapshots, vec![json!({"operation":"insert","actor":"1"})]);
  let rejected = request(
    &s,
    "PATCH",
    &format!("/projects/30/merge-request-approval-rules/{id}"),
    "alice",
    json!({"name":"Cross project"}),
  )
  .await;
  assert_eq!(rejected.0, StatusCode::NOT_FOUND);
  // The fixture database is disposable. Force the audit write to fail after
  // the mutation so this proves that both operations share one transaction.
  s.db
    .execute(
      "DROP TABLE project_merge_request_approval_rule_audits",
      vec![],
    )
    .await
    .unwrap();
  let failed = request(
    &s,
    "PATCH",
    &format!("/projects/20/merge-request-approval-rules/{id}"),
    "alice",
    json!({"name":"Must roll back"}),
  )
  .await;
  assert!(!failed.0.is_success());
  assert_eq!(
    s.collaboration
      .approval_rule(20, id.parse().unwrap())
      .await
      .unwrap()
      .name,
    "Review"
  );
}
