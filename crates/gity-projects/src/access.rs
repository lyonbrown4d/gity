use gity_core::{AppError, Principal};
use gity_entities::{organization_members, organizations, project_members, projects};
use gity_identity::MembershipRole;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, sea_query::Expr};

pub(crate) async fn lock_organization<C: ConnectionTrait>(
  db: &C,
  id: Option<i64>,
) -> Result<(), AppError> {
  let mut update = organizations::Entity::update_many().col_expr(
    organizations::Column::UpdatedAt,
    Expr::col(organizations::Column::UpdatedAt),
  );
  if let Some(id) = id {
    update = update.filter(organizations::Column::Id.eq(id));
  }
  update.exec(db).await?;
  Ok(())
}
pub(crate) async fn organization<C: ConnectionTrait>(
  db: &C,
  actor: &Principal,
  id: i64,
  minimum: i64,
) -> Result<(organizations::Model, String), AppError> {
  let org = organizations::Entity::find_by_id(id)
    .one(db)
    .await?
    .ok_or_else(|| AppError::not_found("organization not found"))?;
  let member = organization_members::Entity::find()
    .filter(organization_members::Column::OrganizationId.eq(id))
    .filter(organization_members::Column::UserId.eq(actor.user_id))
    .one(db)
    .await?;
  let role = if actor.is_super_admin {
    "owner".to_owned()
  } else {
    member.map(|m| m.role).unwrap_or_default()
  };
  let allowed = if minimum == 0 {
    org.visibility != "private" || !role.is_empty()
  } else {
    MembershipRole::parse(&role).level() >= minimum
  };
  if !allowed {
    return Err(AppError::forbidden("insufficient organization permissions"));
  }
  Ok((org, role))
}
pub(crate) async fn project<C: ConnectionTrait>(
  db: &C,
  actor: &Principal,
  id: i64,
  minimum: i64,
) -> Result<(projects::Model, i64), AppError> {
  let model = projects::Entity::find_by_id(id)
    .filter(projects::Column::DeletedAt.is_null())
    .one(db)
    .await?
    .ok_or_else(|| AppError::not_found("project not found"))?;
  let direct = project_members::Entity::find()
    .filter(project_members::Column::ProjectId.eq(id))
    .filter(project_members::Column::UserId.eq(actor.user_id))
    .one(db)
    .await?;
  let org = organization_members::Entity::find()
    .filter(organization_members::Column::OrganizationId.eq(model.organization_id))
    .filter(organization_members::Column::UserId.eq(actor.user_id))
    .one(db)
    .await?;
  let level = if actor.is_super_admin {
    100
  } else {
    direct
      .into_iter()
      .map(|m| MembershipRole::parse(&m.role).level())
      .chain(
        org
          .into_iter()
          .map(|m| MembershipRole::parse(&m.role).level()),
      )
      .max()
      .unwrap_or(0)
  };
  if level < minimum {
    return Err(AppError::forbidden("insufficient project permissions"));
  }
  Ok((model, level))
}
