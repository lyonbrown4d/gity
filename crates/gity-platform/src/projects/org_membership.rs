use super::*;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, PaginatorTrait,
  QueryFilter, Set,
};

pub(super) enum Change<'a> {
  Add(&'a str),
  Set(&'a str),
  Remove,
}

pub(super) async fn mutate(
  s: &AppState,
  headers: &HeaderMap,
  organization: i64,
  user: i64,
  change: Change<'_>,
) -> Reply {
  let actor = auth::principal(s, headers).await?;
  let tx = s.db.transaction().await?;
  // Lock the organization before inspecting any membership. The no-change
  // UPDATE also obtains SQLite's write lock without a deferred-read upgrade.
  tx.execute_raw(s.db.statement(
    "UPDATE organizations SET updated_at=updated_at WHERE id=?",
    vec![organization.into()],
  ))
  .await?;
  gity_entities::organizations::Entity::find_by_id(organization)
    .one(&tx)
    .await?
    .ok_or_else(|| AppError::not_found("organization not found"))?;
  if !actor.is_super_admin {
    let membership = gity_entities::organization_members::Entity::find()
      .filter(gity_entities::organization_members::Column::OrganizationId.eq(organization))
      .filter(gity_entities::organization_members::Column::UserId.eq(actor.user_id))
      .one(&tx)
      .await?;
    if membership.is_none_or(|row| row.role != "owner") {
      return Err(AppError::forbidden("insufficient organization permissions"));
    }
  }
  let existing = gity_entities::organization_members::Entity::find()
    .filter(gity_entities::organization_members::Column::OrganizationId.eq(organization))
    .filter(gity_entities::organization_members::Column::UserId.eq(user))
    .one(&tx)
    .await?;
  if matches!(change, Change::Add(_)) && existing.is_some() {
    return Err(AppError::conflict("organization member already exists"));
  }
  if !matches!(change, Change::Add(_)) && existing.is_none() {
    return Err(AppError::not_found("organization member not found"));
  }
  let keeps_owner = matches!(change, Change::Add("owner") | Change::Set("owner"));
  if !keeps_owner && existing.as_ref().is_some_and(|row| row.role == "owner") {
    let count = gity_entities::organization_members::Entity::find()
      .filter(gity_entities::organization_members::Column::OrganizationId.eq(organization))
      .filter(gity_entities::organization_members::Column::Role.eq("owner"))
      .count(&tx)
      .await?;
    if count <= 1 {
      return Err(AppError::conflict(
        "organization must retain at least one owner",
      ));
    }
  }
  match change {
    Change::Add(role) => {
      gity_entities::users::Entity::find_by_id(user)
        .one(&tx)
        .await?
        .ok_or_else(|| AppError::not_found("user not found"))?;
      let now = chrono::Utc::now().naive_utc();
      gity_entities::organization_members::ActiveModel {
        id: Set(gity_core::new_id()),
        organization_id: Set(organization),
        user_id: Set(user),
        role: Set(role.to_owned()),
        created_at: Set(now),
        updated_at: Set(now),
      }
      .insert(&tx)
      .await?;
    }
    Change::Set(role) => {
      let mut member = existing
        .clone()
        .ok_or_else(|| AppError::not_found("organization member not found"))?
        .into_active_model();
      member.role = Set(role.to_owned());
      member.updated_at = Set(chrono::Utc::now().naive_utc());
      member.update(&tx).await?;
    }
    Change::Remove => {
      let member = existing
        .clone()
        .ok_or_else(|| AppError::not_found("organization member not found"))?;
      gity_entities::organization_members::Entity::delete_by_id(member.id)
        .exec(&tx)
        .await?;
      tx.commit().await?;
      return Ok(ok(json!({"ok":true})));
    }
  }
  let member = gity_entities::organization_members::Entity::find()
    .filter(gity_entities::organization_members::Column::OrganizationId.eq(organization))
    .filter(gity_entities::organization_members::Column::UserId.eq(user))
    .one(&tx)
    .await?
    .ok_or_else(|| AppError::not_found("organization member not found"))?;
  let user_model = gity_entities::users::Entity::find_by_id(user)
    .one(&tx)
    .await?
    .ok_or_else(|| AppError::not_found("user not found"))?;
  let row = json!({
      "id": member.id,
      "organization_id": member.organization_id,
      "user_id": member.user_id,
      "role": member.role,
      "created_at": member.created_at.and_utc().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
      "updated_at": member.updated_at.and_utc().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
      "username": user_model.username,
      "display_name": user_model.display_name,
      "email": user_model.email,
  });
  tx.commit().await?;
  Ok(ok(row))
}
