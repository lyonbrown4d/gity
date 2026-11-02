use super::*;
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    get,path="/projects/{id}/permissions",tag="projects",
    description="Returns effective project/organization role, actions and capabilities for the current user or scoped project token.",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Permissions>),api::Errors),
    security(("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
pub(super) async fn permissions(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(id): Path<i64>,
) -> Reply {
  let authorization = access::project_action(&s, &h, id, access::ProjectAction::Read).await?;
  let principal = authorization.actor()?;
  let level = authorization.role_level();
  let role = if authorization.is_project_token() {
    "project_token"
  } else {
    match level {
      50.. => "owner",
      40.. => "maintainer",
      30.. => "developer",
      20.. => "reporter",
      _ => "guest",
    }
  };
  let mut actions = serde_json::Map::new();
  let mut capabilities = serde_json::Map::new();
  // Preserve the legacy action names and their capability aliases. Read has
  // already passed project visibility and membership authorization above.
  for (action, aliases, operation) in ACTIONS {
    let allowed = json!(authorization.allows(*operation));
    actions.insert((*action).into(), allowed.clone());
    for alias in *aliases {
      capabilities.insert((*alias).into(), allowed.clone());
    }
  }
  Ok(ok(json!({
      "project_id":id.to_string(),
      "user_id":principal.user_id.to_string(),
      "role":role,
      "role_level":level,
      "actions":actions,
      "capabilities":capabilities,
  })))
}

use access::ProjectAction;
const ACTIONS: &[(&str, &[&str], ProjectAction)] = &[
  ("project.read", &["can_read"], ProjectAction::Read),
  (
    "project.write",
    &["can_write", "ci_write", "audit_read"],
    ProjectAction::Write,
  ),
  ("project.delete", &["can_delete"], ProjectAction::Delete),
  (
    "project.repository.read",
    &[],
    ProjectAction::RepositoryRead,
  ),
  (
    "project.repository.push",
    &["repository_push"],
    ProjectAction::RepositoryPush,
  ),
  (
    "project.repository.admin",
    &["repository_admin"],
    ProjectAction::RepositoryAdmin,
  ),
  (
    "project.issues.create",
    &["issue_create"],
    ProjectAction::IssueCreate,
  ),
  (
    "project.issues.write",
    &["issue_write"],
    ProjectAction::IssueWrite,
  ),
  (
    "project.issues.comment",
    &["issue_comment"],
    ProjectAction::IssueComment,
  ),
  (
    "project.merge_requests.create",
    &["merge_request_create"],
    ProjectAction::MergeRequestCreate,
  ),
  (
    "project.merge_requests.write",
    &["merge_request_write"],
    ProjectAction::MergeRequestWrite,
  ),
  (
    "project.merge_requests.comment",
    &["merge_request_comment"],
    ProjectAction::MergeRequestComment,
  ),
  (
    "project.merge_requests.merge",
    &["merge_request_merge"],
    ProjectAction::MergeRequestMerge,
  ),
  ("project.packages.read", &[], ProjectAction::PackageRead),
  (
    "project.packages.write",
    &["package_write"],
    ProjectAction::PackageWrite,
  ),
  ("project.wiki.read", &[], ProjectAction::WikiRead),
  (
    "project.wiki.write",
    &["wiki_write"],
    ProjectAction::WikiWrite,
  ),
  (
    "project.jobs.read",
    &["ci_read", "job_read"],
    ProjectAction::JobRead,
  ),
  (
    "project.jobs.write",
    &["job_write"],
    ProjectAction::JobWrite,
  ),
  (
    "project.runners.read",
    &["runner_read"],
    ProjectAction::RunnerRead,
  ),
  (
    "project.runners.admin",
    &["runner_admin"],
    ProjectAction::RunnerAdmin,
  ),
];
