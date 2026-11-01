use super::merge_requests::{git_error, repository};
use super::*;

pub(super) async fn merged(
    s: &AppState,
    project: &Value,
    row: &Value,
    actor: &crate::access::ProjectActor,
) -> Result<(), AppError> {
    let p = number(project, "id");
    let i = number(row, "iid");
    let record = crate::registry::AuditRecord {
        project: p,
        actor: actor.user_id,
        event: "project.merge_request.merged",
        action: "merge_merge_request",
        target_type: "merge_request",
        target_id: &number(row, "id").to_string(),
        summary: &format!(
            "Merge request !{i} merged into {}",
            string(row, "target_branch")
        ),
        payload: json!({
            "project_id": p,
            "merge_request_id": number(row, "id"),
            "merge_iid": i,
            "actor_user_id": actor.user_id,
            "source_branch": row["source_branch"],
            "target_branch": row["target_branch"],
        }),
    };
    if let Err(error) = crate::registry::record_event(s, record).await {
        tracing::warn!(project_id=p,error=%error,"merge audit event persistence failed");
    }
    let target = string(row, "target_branch");
    let target_sha = s
        .git
        .resolve_revision(
            &repository(project),
            target,
            string(project, "default_branch"),
        )
        .await
        .map_err(git_error)?;
    if let Err(error) = crate::ci::create_push_pipeline(s, p, target, &target_sha).await {
        tracing::warn!(project_id=p,branch=target,error=%error,"merged target pipeline creation failed");
    }
    let user = number(row, "author_user_id");
    if user != actor.user_id {
        super::todos::notify(
            s,
            super::todos::Notification {
                project: p,
                user,
                kind: "merge_request_merged",
                target_type: "merge_request",
                target: number(row, "id"),
                title: &format!("Merge request !{i} merged"),
                summary: string(row, "title"),
                suffix: "?tab=merge-requests",
            },
        )
        .await?;
    }
    Ok(())
}
