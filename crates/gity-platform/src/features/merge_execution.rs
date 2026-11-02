use super::*;

pub(super) async fn merge(s: &AppState, h: &HeaderMap, p: i64, i: i64, b: &Value) -> Reply {
  let authorization =
    crate::access::project_action(s, h, p, ProjectAction::MergeRequestMerge).await?;
  let actor = authorization.actor()?;
  let project = authorization.project;
  let _guard = lock(p).await;
  let row = model_value(s.collaboration.merge_request(p, i).await?);
  // Reconcile proven earlier execution before current branch/CI state can
  // prevent recording it. This does not authorize a new Git mutation.
  let existing = s.collaboration.active_merge_operation(p, i).await?;
  if let Some(op) = &existing
    && let Some(completed) = super::super::merge_events::reconcile(s, op.id).await?
  {
    return complete(s, p, i, completed).await;
  }
  if string(&row, "state") != "opened" {
    return Err(AppError::conflict("merge request is not opened"));
  }
  let checks = super::super::checks::evaluate(s, &project, &row).await?;
  if checks["mergeable"] != true {
    return Err(AppError::conflict(string(&checks, "blocking_reason")));
  }
  if string(&checks, "merge_access_level") == "owner" {
    crate::access::project(s, h, p, 50).await?;
  }
  if string(&checks, "merge_access_level") == "no_one" {
    return Err(AppError::forbidden("target branch does not permit merges"));
  }
  let target_sha = branch(s, &project, string(&row, "target_branch")).await?;
  let op = match existing {
    Some(op) => op,
    None => {
      s.collaboration
        .claim_merge_operation(gity_collaboration::MergeOperationClaim {
          operation_key: uuid::Uuid::now_v7().to_string(),
          project_id: p,
          iid: i,
          actor_user_id: actor.user_id,
          source_sha: string(&checks, "source_commit_sha").into(),
          target_sha,
          message: if string(b, "message").is_empty() {
            format!(
              "Merge branch '{}' into '{}'",
              string(&row, "source_branch"),
              string(&row, "target_branch")
            )
          } else {
            string(b, "message").into()
          },
          author_name: if string(b, "author_name").is_empty() {
            actor.username.clone()
          } else {
            string(b, "author_name").into()
          },
          author_email: if string(b, "author_email").is_empty() {
            format!("{}@gity.local", actor.username)
          } else {
            string(b, "author_email").into()
          },
        })
        .await?
    }
  };
  // Serialize Git execution and completion with recovery/abort on every node.
  // The claim was committed before taking this guard, so Git never precedes
  // the durable journal. No other DB connection is used while it is held.
  let completed = super::super::merge_events::apply(s, op.id).await?;
  complete(s, p, i, completed).await
}

async fn complete(
  s: &AppState,
  p: i64,
  i: i64,
  op: gity_entities::merge_operations::Model,
) -> Reply {
  if let Err(error) = super::super::merge_events::deliver(s, &op).await {
    tracing::warn!(operation_id=op.id, error=%error, "merge completion outbox pending");
  }
  if let Some(reason) = op.rejection {
    return Err(AppError::conflict(reason));
  }
  response(view(model_value(
    s.collaboration.merge_request(p, i).await?,
  )))
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub(super) struct AbortMergeOperation {
  /// Human reason, persisted with the owner actor in the immutable receipt.
  reason: String,
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
  post, path="/{id}/merge-requests/{iid}/merge/abort", tag="merge requests",
  params(api::IidPath), request_body(content=AbortMergeOperation),
  responses((status=200, description="Merge reconciled or explicitly aborted", body=api::Envelope<spec::MergeRequest>),api::Errors),
  security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
pub(super) async fn abort(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
  Json(command): Json<AbortMergeOperation>,
) -> Reply {
  let authorization =
    crate::access::project_action(&s, &h, p, ProjectAction::MergeRequestMerge).await?;
  let actor = authorization.actor()?;
  crate::access::project(&s, &h, p, 50).await?;
  if command.reason.trim().is_empty() {
    return Err(AppError::bad_request("abort reason is required"));
  }
  let _guard = lock(p).await;
  let op = s
    .collaboration
    .active_merge_operation(p, i)
    .await?
    .ok_or_else(|| AppError::conflict("no pending merge operation"))?;
  let reason = format!("owner {}: {}", actor.user_id, command.reason.trim());
  let op = super::super::merge_events::abort_operation(&s, op.id, reason, actor.user_id).await?;
  if let Err(error) = super::super::merge_events::deliver(&s, &op).await {
    tracing::warn!(operation_id=op.id, error=%error, "merge completion outbox pending");
  }
  response(view(model_value(
    s.collaboration.merge_request(p, i).await?,
  )))
}
