use gity_core::AppError;
use gity_entities::{
  merge_operations, project_audit_events as events, project_merge_requests, project_todos as todos,
  projects,
};
use sea_orm::{ActiveModelTrait, DatabaseTransaction, EntityTrait, Set, sea_query::OnConflict};

pub(crate) async fn record(
  tx: &DatabaseTransaction,
  op: &merge_operations::Model,
  mr: &project_merge_requests::Model,
  sha: &str,
) -> Result<(), AppError> {
  let project = projects::Entity::find_by_id(op.project_id)
    .one(tx)
    .await?
    .ok_or_else(|| AppError::not_found("project not found"))?;
  let now = chrono::Utc::now().naive_utc();
  events::ActiveModel {
    id: Set(gity_core::new_id()),
    project_id: Set(project.id),
    organization_id: Set(project.organization_id),
    event_name: Set("project.merge_request.merged".into()),
    action: Set("merge_merge_request".into()),
    actor_user_id: Set(op.actor_user_id),
    target_type: Set("merge_request".into()),
    target_id: Set(mr.id.to_string()),
    summary: Set(format!(
      "Merge request !{} merged into {}",
      mr.iid, mr.target_branch
    )),
    payload: Set(
      serde_json::json!({
        "project_id": project.id.to_string(), "merge_request_id": mr.id.to_string(),
        "merge_iid": mr.iid, "actor_user_id": op.actor_user_id.to_string(),
        "source_branch": op.source_branch, "target_branch": op.target_branch,
        "merge_commit_sha": sha, "operation_key": op.operation_key,
      })
      .to_string(),
    ),
    created_at: Set(now),
  }
  .insert(tx)
  .await?;
  if mr.author_user_id == op.actor_user_id {
    return Ok(());
  }
  todos::Entity::insert(todos::ActiveModel {
    id: Set(gity_core::new_id()),
    user_id: Set(mr.author_user_id),
    organization_id: Set(project.organization_id),
    project_id: Set(project.id),
    kind: Set("merge_request_merged".into()),
    state: Set("pending".into()),
    target_type: Set("merge_request".into()),
    target_id: Set(mr.id.to_string()),
    title: Set(format!("Merge request !{} merged", mr.iid)),
    summary: Set(mr.title.clone()),
    action_url: Set(format!(
      "/app/projects/{}/{}?tab=merge-requests",
      project.organization_id, project.id
    )),
    created_at: Set(now),
    updated_at: Set(now),
    done_at: Set(None),
  })
  .on_conflict(
    OnConflict::columns([
      todos::Column::UserId,
      todos::Column::Kind,
      todos::Column::TargetType,
      todos::Column::TargetId,
    ])
    .do_nothing_on([todos::Column::Id])
    .to_owned(),
  )
  .try_insert()
  .exec(tx)
  .await?;
  Ok(())
}

pub(crate) async fn rejected(
  tx: &DatabaseTransaction,
  op: &merge_operations::Model,
  reason: &str,
  actor_user_id: i64,
) -> Result<(), AppError> {
  let project = projects::Entity::find_by_id(op.project_id)
    .one(tx)
    .await?
    .ok_or_else(|| AppError::not_found("project not found"))?;
  events::ActiveModel {
    id: Set(gity_core::new_id()),
    project_id: Set(project.id),
    organization_id: Set(project.organization_id),
    event_name: Set("project.merge_operation.rejected".into()),
    action: Set("reject_merge_operation".into()),
    actor_user_id: Set(actor_user_id),
    target_type: Set("merge_request".into()),
    target_id: Set(op.merge_request_id.to_string()),
    summary: Set("Merge operation rejected or explicitly aborted".into()),
    payload: Set(
      serde_json::json!({"operation_key": op.operation_key, "reason": reason, "original_merge_actor_user_id": op.actor_user_id.to_string()}).to_string(),
    ),
    created_at: Set(chrono::Utc::now().naive_utc()),
  }
  .insert(tx)
  .await?;
  Ok(())
}
