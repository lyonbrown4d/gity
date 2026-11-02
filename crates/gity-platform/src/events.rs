use crate::{AppState, features, projects, registry, repository};
use gity_core::AppError;
use serde_json::{Value, json};

pub enum ProjectChange<'a> {
  Created,
  Deleted,
  Protection { branch: &'a str, enabled: bool },
  BranchDeleted(&'a str),
}
pub async fn project_change(
  s: &AppState,
  p: &Value,
  actor: i64,
  change: ProjectChange<'_>,
) -> Result<(), AppError> {
  let id = projects::id(&p["id"])?;
  let (event, action, kind, target_type, target, title, summary, suffix) = match change {
    ProjectChange::Created => (
      "project.created",
      "create_project",
      "project_created",
      "project",
      id.to_string(),
      format!(
        "Project created: {}",
        p["full_path"].as_str().unwrap_or_default()
      ),
      "A new project is ready for collaboration.".to_owned(),
      "",
    ),
    ProjectChange::Deleted => (
      "project.deleted",
      "delete_project",
      "",
      "project",
      id.to_string(),
      String::new(),
      "Project deleted".to_owned(),
      "",
    ),
    ProjectChange::Protection { branch, enabled } => (
      "project.branch_protection.changed",
      if enabled {
        "protect_branch"
      } else {
        "unprotect_branch"
      },
      "branch_protection_changed",
      "branch_protection",
      branch.to_owned(),
      format!("Branch protection changed: {branch}"),
      format!(
        "Branch protection {} for {branch}",
        if enabled { "updated" } else { "removed" }
      ),
      "?tab=settings",
    ),
    ProjectChange::BranchDeleted(branch) => (
      "project.branch.deleted",
      "delete_branch",
      "branch_deleted",
      "branch",
      branch.to_owned(),
      format!("Branch deleted: {branch}"),
      format!("Branch {branch} deleted"),
      "?tab=branches",
    ),
  };
  registry::record_event(
    s,
    registry::AuditRecord {
      project: id,
      actor,
      event,
      action,
      target_type,
      target_id: &target,
      summary: &summary,
      payload: json!({"project_id":id,"target":target}),
    },
  )
  .await?;
  if kind.is_empty() {
    return Ok(());
  }
  let org = projects::id(&p["organization_id"])?;
  let recipients = s
    .projects
    .member_user_ids(id, org, kind == "project_created")
    .await?;
  for recipient in recipients {
    features::notify_event(
      s,
      features::Notification {
        project: id,
        user: recipient,
        kind,
        target_type,
        target: &target,
        title: &title,
        summary: &summary,
        suffix,
      },
    )
    .await?;
  }
  Ok(())
}

/// A Git mutation has already succeeded. Follow-up failures are observable and
/// must not report that the accepted commit itself failed.
pub async fn repository_changed(s: &AppState, p: &Value, reference: &str, sha: &str) {
  let Ok(id) = projects::id(&p["id"]) else {
    return;
  };
  if let Err(error) = crate::ci::create_push_pipeline(s, id, reference, sha).await {
    tracing::error!(error=%error,project_id=id,reference,"push pipeline creation failed");
  }
  match crate::code_index::enqueue(s, p, reference, sha).await {
    Ok(Some(_)) => {
      metrics::counter!("gity_code_index_enqueue_total", "result" => "success").increment(1);
    }
    Ok(None) => {}
    Err(error) => {
      metrics::counter!("gity_code_index_enqueue_total", "result" => "failure").increment(1);
      tracing::warn!(error=%error,project_id=id,reference,"code index job enqueue failed");
    }
  }
  let branch = reference.strip_prefix("refs/heads/").unwrap_or(reference);
  if s.settings.search.index_enabled && branch == repository::default_branch(p) {
    let state = s.clone();
    let project = p.clone();
    tokio::spawn(async move {
      if let Err(error) = state
        .search
        .refresh(
          id,
          &repository::repo(&project),
          repository::default_branch(&project),
        )
        .await
      {
        tracing::warn!(error=%error,project_id=id,"search refresh after Git mutation failed");
      }
    });
  }
}
