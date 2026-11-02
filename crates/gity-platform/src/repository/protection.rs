use super::*;
use crate::openapi::{repository_schemas as spec, schemas as api};

pub fn matches_rule(pattern: &str, branch: &str) -> bool {
  glob::Pattern::new(pattern).is_ok_and(|p| p.matches(branch))
}
pub fn effective_rule<'a>(rules: &'a [Value], branch: &str) -> Option<&'a Value> {
  let exact = |v: &&Value| v["branch_name"] == branch && v["rule_type"] != "pattern";
  rules.iter().find(exact).or_else(|| {
    rules
      .iter()
      .filter(|v| {
        v["rule_type"] != "exact"
          && matches_rule(v["branch_name"].as_str().unwrap_or_default(), branch)
      })
      .min_by_key(|v| v["branch_name"].as_str().unwrap_or_default())
  })
}
pub async fn check_branch(
  s: &AppState,
  h: &HeaderMap,
  p: &Value,
  branch: &str,
  deleting: bool,
  merging: bool,
) -> Result<(), AppError> {
  let authorization = access::project_action(
    s,
    h,
    projects::id(&p["id"])?,
    access::ProjectAction::RepositoryPush,
  )
  .await?;
  let level = if authorization.is_project_token() {
    30
  } else {
    authorization.role_level()
  };
  let rules = s
    .repository_access
    .protections(projects::id(&p["id"])?)
    .await?
    .into_iter()
    .map(gity_persistence::model_value)
    .collect::<Vec<_>>();
  if let Some(rule) = effective_rule(&rules, branch) {
    let yes = |k: &str| {
      rule[k]
        .as_bool()
        .unwrap_or_else(|| rule[k].as_i64() == Some(1))
    };
    if deleting && !yes("allow_delete") {
      return Err(AppError::forbidden("protected branch cannot be deleted"));
    }
    if !deleting && !merging && yes("require_merge_request") {
      return Err(AppError::forbidden(
        "protected branch requires a merge request",
      ));
    }
    let access = rule[if merging {
      "merge_access_level"
    } else {
      "push_access_level"
    }]
    .as_str()
    .unwrap_or("no_one");
    let minimum = match access {
      "developer" | "developers" => 30,
      "maintainer" | "maintainers" => 40,
      "owner" | "owners" => 50,
      _ => i64::MAX,
    };
    if level < minimum {
      return Err(AppError::forbidden(
        "protected branch role requirement not met",
      ));
    }
  }
  Ok(())
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects/{id}/repository")]
#[utoipa::path(post,path="/branches/{branch}/protect",tag="Repository",params(spec::BranchPath),request_body=spec::ProtectionInput,
    responses((status=200,description="Create or update branch protection; maintainer required",body=api::Envelope<spec::BranchProtection>),api::Errors),security(("bearer"=[]),("private_token"=[])))]
pub(super) async fn protect(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((id, branch)): Path<(i64, String)>,
  Json(b): Json<Value>,
) -> Reply {
  let project = access::project_action(&s, &h, id, access::ProjectAction::RepositoryAdmin)
    .await?
    .project;
  if branch.is_empty()
    || branch.len() > 255
    || (branch.contains(['*', '?', '[']) && glob::Pattern::new(&branch).is_err())
  {
    return Err(AppError::bad_request("invalid branch pattern"));
  }
  let mut row=json!({"project_id":id,"branch_name":branch,"rule_type":if branch.contains(['*','?','[']){"pattern"}else{"exact"},"push_access_level":"no_one","merge_access_level":"maintainer","require_merge_request":true,"require_pipeline_success":false,"allow_force_push":false,"allow_delete":false}).as_object().unwrap().clone();
  for key in [
    "push_access_level",
    "merge_access_level",
    "require_merge_request",
    "require_pipeline_success",
    "allow_force_push",
    "allow_delete",
  ] {
    if let Some(v) = b.get(key) {
      if key != "push_access_level" && key != "merge_access_level" && !v.is_boolean() {
        return Err(AppError::bad_request("protection flags must be booleans"));
      }
      row.insert(key.into(), v.clone());
    }
  }
  for key in ["push_access_level", "merge_access_level"] {
    if !["maintainer", "developer", "owner", "no_one"]
      .contains(&row[key].as_str().unwrap_or_default())
    {
      return Err(AppError::bad_request(
        "invalid protected branch access level",
      ));
    }
  }
  let command = gity_projects::ProtectionCommand {
    rule_type: row["rule_type"].as_str().unwrap_or("exact").to_owned(),
    push_access_level: row["push_access_level"]
      .as_str()
      .unwrap_or("no_one")
      .to_owned(),
    merge_access_level: row["merge_access_level"]
      .as_str()
      .unwrap_or("maintainer")
      .to_owned(),
    require_merge_request: row["require_merge_request"].as_bool().unwrap_or(true),
    require_pipeline_success: row["require_pipeline_success"].as_bool().unwrap_or(false),
    allow_force_push: row["allow_force_push"].as_bool().unwrap_or(false),
    allow_delete: row["allow_delete"].as_bool().unwrap_or(false),
    actor: gity_core::audit_actor(),
  };
  let row = gity_persistence::model_value(
    s.repository_access
      .save_protection(id, &branch, &command)
      .await?,
  );
  let actor = auth::principal(&s, &h).await?;
  crate::events::project_change(
    &s,
    &project,
    actor.user_id,
    crate::events::ProjectChange::Protection {
      branch: &branch,
      enabled: true,
    },
  )
  .await?;
  Ok(ok(views::protection(row)))
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects/{id}/repository")]
#[utoipa::path(post,path="/branches/{branch}/unprotect",tag="Repository",params(spec::BranchPath),
    responses((status=200,description="Remove protection rule; maintainer required",body=api::Envelope<spec::Ack>),api::Errors),security(("bearer"=[]),("private_token"=[])))]
pub(super) async fn unprotect(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((id, branch)): Path<(i64, String)>,
) -> Reply {
  let project = access::project_action(&s, &h, id, access::ProjectAction::RepositoryAdmin)
    .await?
    .project;
  if s.repository_access.delete_protection(id, &branch).await? {
    let actor = auth::principal(&s, &h).await?;
    crate::events::project_change(
      &s,
      &project,
      actor.user_id,
      crate::events::ProjectChange::Protection {
        branch: &branch,
        enabled: false,
      },
    )
    .await?;
  }
  Ok(ok(json!({"ok":true})))
}

#[gity_api_macros::application_operation(prefix = "/api/v1/projects/{id}/repository")]
#[utoipa::path(patch,path="/branch-protections/{branch}",tag="Repository",params(spec::BranchPath),request_body=spec::ProtectionInput,
    responses((status=200,description="Create or update branch protection; maintainer required",body=api::Envelope<spec::BranchProtection>),api::Errors),security(("bearer"=[]),("private_token"=[])))]
pub(super) async fn update(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(path): Path<(i64, String)>,
  Json(body): Json<Value>,
) -> Reply {
  protect(State(s), h, Path(path), Json(body)).await
}
