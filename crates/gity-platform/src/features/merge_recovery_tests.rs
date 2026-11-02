use super::super::test_support::{fixture, request};
use super::*;
use gity_collaboration::{CollaborationService, MergeOperationClaim};

pub(super) async fn prepared(s: &AppState) -> merge_operations::Model {
  s.git.init_bare("20.git", "main").await.unwrap();
  let target = s
    .git
    .create_file_commit(
      "20.git",
      super::super::merge_tests::file("main", "base", "base"),
    )
    .await
    .unwrap();
  s.git
    .create_branch("20.git", "feature", "main")
    .await
    .unwrap();
  let source = s
    .git
    .create_file_commit(
      "20.git",
      super::super::merge_tests::file("feature", "feature", "feature"),
    )
    .await
    .unwrap();
  let created = request(
    s,
    "POST",
    "/projects/20/merge-requests",
    "alice",
    json!({"title":"Feature", "source_branch":"feature", "target_branch":"main"}),
  )
  .await;
  assert_eq!(created.0, axum::http::StatusCode::OK);
  s.collaboration
    .claim_merge_operation(claim(source, target))
    .await
    .unwrap()
}
fn claim(source: String, target: String) -> MergeOperationClaim {
  MergeOperationClaim {
    operation_key: uuid::Uuid::now_v7().to_string(),
    project_id: 20,
    iid: 1,
    actor_user_id: 2,
    source_sha: source,
    target_sha: target,
    message: "merge feature".into(),
    author_name: "Bob".into(),
    author_email: "bob@example.test".into(),
  }
}

#[tokio::test]
async fn git_success_database_failure_restart_recovers_exact_sha_and_one_event() {
  let (_dir, s) = fixture().await;
  let op = prepared(&s).await;
  let outcome = s.git.merge_operation("20.git", input(&op)).await.unwrap();
  let MergeOperationOutcome::Applied { commit_sha } = &outcome else {
    panic!("merge rejected")
  };
  let commit_sha = commit_sha.clone();
  // Inject failure inside the completion transaction after Git has succeeded.
  // Test-only DDL keeps production code free of generic fault injection paths.
  s.db
    .execute(
      "ALTER TABLE project_audit_events RENAME TO unavailable_audit_events",
      vec![],
    )
    .await
    .unwrap();
  assert!(finish(&s, op.id, outcome).await.is_err());
  assert_eq!(
    s.collaboration.merge_request(20, 1).await.unwrap().state,
    "opened"
  );
  assert_eq!(
    s.collaboration
      .active_merge_operation(20, 1)
      .await
      .unwrap()
      .unwrap()
      .id,
    op.id
  );
  s.db
    .execute(
      "ALTER TABLE unavailable_audit_events RENAME TO project_audit_events",
      vec![],
    )
    .await
    .unwrap();
  let newer = s
    .git
    .create_file_commit(
      "20.git",
      super::super::merge_tests::file("main", "later", "later"),
    )
    .await
    .unwrap();
  assert_ne!(commit_sha, newer);
  // Reconstruct the service and revoke the actor: recovery only reads proof.
  s.db
    .execute(
      "DELETE FROM project_members WHERE project_id=20 AND user_id=2",
      vec![],
    )
    .await
    .unwrap();
  let mut restarted = s.clone();
  restarted.collaboration = fluxdi::Shared::new(CollaborationService::new(s.db.connection.clone()));
  recover(&restarted).await.unwrap();
  recover(&restarted).await.unwrap();
  let mr = restarted.collaboration.merge_request(20, 1).await.unwrap();
  assert_eq!(mr.merge_commit_sha.as_deref(), Some(commit_sha.as_str()));
  assert_eq!(mr.state, "merged");
  let events = s
    .db
    .all(
      "SELECT id FROM project_audit_events WHERE event_name='project.merge_request.merged'",
      vec![],
    )
    .await
    .unwrap();
  assert_eq!(events.len(), 1);
  let todos = s
    .db
    .all(
      "SELECT id FROM project_todos WHERE kind='merge_request_merged'",
      vec![],
    )
    .await
    .unwrap();
  assert_eq!(todos.len(), 1);
}

#[tokio::test]
async fn pending_without_git_proof_is_not_executed_by_worker_and_fresh_policy_blocks_retry() {
  let (_dir, s) = fixture().await;
  let op = prepared(&s).await;
  recover(&s).await.unwrap();
  assert_eq!(
    s.git.resolve_revision("20.git", "main", "").await.unwrap(),
    op.target_sha
  );
  s.db.insert("project_branch_protections", fields(json!({"project_id":20, "branch_name":"main", "rule_type":"exact", "push_access_level":"maintainer", "merge_access_level":"no_one", "allow_force_push":0,"allow_delete":0,"require_merge_request":0,"require_pipeline_success":0}))).await.unwrap();
  let retry = request(
    &s,
    "POST",
    "/projects/20/merge-requests/1/merge",
    "bob",
    json!({}),
  )
  .await;
  assert_eq!(retry.0, axum::http::StatusCode::FORBIDDEN);
  assert!(
    s.collaboration
      .active_merge_operation(20, 1)
      .await
      .unwrap()
      .is_some()
  );
  assert_eq!(
    s.git.resolve_revision("20.git", "main", "").await.unwrap(),
    op.target_sha
  );
}

#[tokio::test]
async fn two_application_instances_claim_same_operation_and_only_one_completion() {
  let (_dir, s) = fixture().await;
  let op = prepared(&s).await;
  let a = CollaborationService::new(s.db.connection.clone());
  let b = CollaborationService::new(s.db.connection.clone());
  let (left, right) = tokio::join!(
    a.claim_merge_operation(claim(op.source_sha.clone(), op.target_sha.clone())),
    b.claim_merge_operation(claim(op.source_sha.clone(), op.target_sha.clone()))
  );
  assert_eq!(left.unwrap().id, op.id);
  assert_eq!(right.unwrap().id, op.id);
  let outcome = s.git.merge_operation("20.git", input(&op)).await.unwrap();
  let MergeOperationOutcome::Applied { commit_sha } = outcome else {
    panic!("merge rejected")
  };
  let (left, right) = tokio::join!(
    a.finish_merge_operation(op.id, Ok(commit_sha.clone())),
    b.finish_merge_operation(op.id, Ok(commit_sha))
  );
  assert_eq!(left.unwrap().status, "applied");
  assert_eq!(right.unwrap().status, "applied");
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
async fn owner_abort_is_audited_and_fences_stale_attempts() {
  let (_dir, s) = fixture().await;
  let op = prepared(&s).await;
  let rejected = request(
    &s,
    "POST",
    "/projects/20/merge-requests/1/merge/abort",
    "bob",
    json!({"reason":"cancel"}),
  )
  .await;
  assert_eq!(rejected.0, axum::http::StatusCode::FORBIDDEN);
  assert!(
    s.collaboration
      .active_merge_operation(20, 1)
      .await
      .unwrap()
      .is_some()
  );
  s.db
    .execute(
      "UPDATE project_members SET role='owner' WHERE project_id=20 AND user_id=1",
      vec![],
    )
    .await
    .unwrap();
  let aborted = request(
    &s,
    "POST",
    "/projects/20/merge-requests/1/merge/abort",
    "alice",
    json!({"reason":"reviewed interrupted attempt; cancel"}),
  )
  .await;
  assert_eq!(aborted.0, axum::http::StatusCode::OK, "{}", aborted.1);
  assert_eq!(aborted.1["data"]["state"], "opened");
  assert!(
    s.collaboration
      .active_merge_operation(20, 1)
      .await
      .unwrap()
      .is_none()
  );
  let result = s.git.merge_operation("20.git", input(&op)).await.unwrap();
  assert!(matches!(
    result,
    MergeOperationOutcome::Rejected {
      actor_user_id: 1,
      ..
    }
  ));
  assert_eq!(
    s.git.resolve_revision("20.git", "main", "").await.unwrap(),
    op.target_sha
  );
  let audit = s.db.one("SELECT actor_user_id, payload FROM project_audit_events WHERE event_name='project.merge_operation.rejected'", vec![]).await.unwrap().unwrap();
  assert_eq!(number(&audit, "actor_user_id"), 1);
  assert!(
    audit["payload"]
      .as_str()
      .unwrap()
      .contains("reviewed interrupted attempt")
  );
  let next = request(
    &s,
    "POST",
    "/projects/20/merge-requests/1/merge",
    "alice",
    json!({}),
  )
  .await;
  assert_eq!(next.0, axum::http::StatusCode::OK, "{}", next.1);
  assert_eq!(next.1["data"]["state"], "merged");
}
