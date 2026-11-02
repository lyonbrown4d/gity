use super::*;
use crate::openapi::{project_schemas as spec, schemas as api};

use std::collections::HashMap;

#[utoipa::path(
    get,path="/projects/{id}/members",tag="projects",
    description="Requires project read access.",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::ProjectMember>>),api::Errors),
    security(("bearer"=[]))
)]
pub(super) async fn members(State(s): State<AppState>, h: HeaderMap, Path(id): Path<i64>) -> Reply {
  access::project_action(&s, &h, id, access::ProjectAction::RepositoryRead).await?;
  let members = s.projects.members(id).await?;
  let user_ids = members
    .iter()
    .map(|member| member.user_id)
    .collect::<Vec<_>>();
  let users = s
    .projects
    .users(user_ids)
    .await?
    .into_iter()
    .map(|user| (user.id, user))
    .collect::<HashMap<_, _>>();
  let mut rows = Vec::with_capacity(members.len());
  for member in members {
    let user = users
      .get(&member.user_id)
      .ok_or_else(|| AppError::not_found("user not found"))?;
    rows.push(member_json(
      &member,
      Some(&user.username),
      Some(&user.display_name),
    ));
  }
  Ok(ok(rows))
}

pub(super) fn member_role(b: &Value) -> Result<&str, AppError> {
  let role = required(b, "role")?;
  if access::role_level(role) == 0 {
    return Err(AppError::bad_request("invalid membership role"));
  }
  Ok(role)
}

#[utoipa::path(
    post,path="/projects/{id}/members",tag="projects",
    description="Requires maintainer. Granting/changing owner membership requires owner.",
    params(api::ProjectPath),
    request_body=spec::AddMember,
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::ProjectMember>),api::Errors),
    security(("bearer"=[]))
)]
pub(super) async fn add_member(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(project): Path<i64>,
  Json(b): Json<Value>,
) -> Reply {
  set_member(State(s), h, Path((project, id(&b["user_id"])?)), Json(b)).await
}

#[utoipa::path(
    patch,path="/projects/{id}/members/{user}",tag="projects",
    description="Requires maintainer; owner membership is protected.",
    params(spec::MemberPath),
    request_body=spec::SetMember,
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::ProjectMember>),api::Errors),
    security(("bearer"=[]))
)]
pub(super) async fn set_member(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((project, user)): Path<(i64, i64)>,
  Json(b): Json<Value>,
) -> Reply {
  access::project(&s, &h, project, 40).await?;
  let actor = auth::principal(&s, &h).await?;
  let member = s
    .projects
    .set_member(&actor, project, user, member_role(&b)?)
    .await?;
  Ok(ok(member_json(&member, None, None)))
}

#[utoipa::path(
    delete,path="/projects/{id}/members/{user}",tag="projects",
    description="Requires maintainer; removing an owner requires owner.",
    params(spec::MemberPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<crate::openapi::user_schemas::OkResult>),api::Errors),
    security(("bearer"=[]))
)]
pub(super) async fn delete_member(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((project, user)): Path<(i64, i64)>,
) -> Reply {
  access::project(&s, &h, project, 40).await?;
  let actor = auth::principal(&s, &h).await?;
  s.projects.delete_member(&actor, project, user).await?;
  Ok(ok(json!({"ok":true})))
}

fn member_json(
  member: &gity_entities::project_members::Model,
  username: Option<&str>,
  display_name: Option<&str>,
) -> Value {
  json!({
      "id": member.id,
      "project_id": member.project_id,
      "user_id": member.user_id,
      "role": member.role,
      "created_at": member.created_at.and_utc().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
      "updated_at": member.updated_at.and_utc().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
      "username": username,
      "display_name": display_name,
  })
}
