use super::*;
use crate::openapi::{project_schemas as spec, schemas as api};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, Set,
};
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
  let members = gity_entities::project_members::Entity::find()
    .filter(gity_entities::project_members::Column::ProjectId.eq(id))
    .order_by_asc(gity_entities::project_members::Column::Id)
    .all(&s.db.connection)
    .await?;
  let user_ids = members
    .iter()
    .map(|member| member.user_id)
    .collect::<Vec<_>>();
  let users = gity_entities::users::Entity::find()
    .filter(gity_entities::users::Column::Id.is_in(user_ids))
    .all(&s.db.connection)
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
  let role = member_role(&b)?;
  let actor = auth::principal(&s, &h).await?;
  let project_row = access::project(&s, &h, project, 40).await?;
  let current = gity_entities::project_members::Entity::find()
    .filter(gity_entities::project_members::Column::ProjectId.eq(project))
    .filter(gity_entities::project_members::Column::UserId.eq(user))
    .one(&s.db.connection)
    .await?;
  if (role == "owner" || current.as_ref().is_some_and(|row| row.role == "owner"))
    && access::level(&s, &actor, &project_row).await? < 50
  {
    return Err(AppError::forbidden("owner role required"));
  }
  let tx = s.db.transaction().await?;
  let current = gity_entities::project_members::Entity::find()
    .filter(gity_entities::project_members::Column::ProjectId.eq(project))
    .filter(gity_entities::project_members::Column::UserId.eq(user))
    .one(&tx)
    .await?;
  let (member, operation) = if let Some(current) = current {
    let mut active = current.into_active_model();
    active.role = Set(role.to_owned());
    active.updated_at = Set(chrono::Utc::now().naive_utc());
    (active.update(&tx).await?, "update")
  } else {
    let timestamp = chrono::Utc::now().naive_utc();
    let member = gity_entities::project_members::ActiveModel {
      id: Set(gity_core::new_id()),
      project_id: Set(project),
      user_id: Set(user),
      role: Set(role.to_owned()),
      created_at: Set(timestamp),
      updated_at: Set(timestamp),
    }
    .insert(&tx)
    .await?;
    (member, "insert")
  };
  let row = member_json(&member, None, None);
  s.db
    .audit_snapshot(&tx, "project_members", &row, operation, Some(actor.user_id))
    .await?;
  tx.commit().await?;
  Ok(ok(row))
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
  let current = gity_entities::project_members::Entity::find()
    .filter(gity_entities::project_members::Column::ProjectId.eq(project))
    .filter(gity_entities::project_members::Column::UserId.eq(user))
    .one(&s.db.connection)
    .await?;
  if let Some(current) = current {
    let actor = auth::principal(&s, &h).await?;
    if current.role == "owner" {
      let project_row = access::project(&s, &h, project, 40).await?;
      if access::level(&s, &actor, &project_row).await? < 50 {
        return Err(AppError::forbidden("owner role required"));
      }
    }
    let row = member_json(&current, None, None);
    let tx = s.db.transaction().await?;
    gity_entities::project_members::Entity::delete_by_id(current.id)
      .exec(&tx)
      .await?;
    s.db
      .audit_snapshot(&tx, "project_members", &row, "delete", Some(actor.user_id))
      .await?;
    tx.commit().await?;
  }
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
