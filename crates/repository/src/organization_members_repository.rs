use chrono::Utc;
use entity::organization_members;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DbErr, EntityTrait, IntoActiveModel,
  QueryFilter, QueryOrder, Set, prelude::DateTimeWithTimeZone,
};

pub struct OrganizationMembersRepository;

impl OrganizationMembersRepository {
  pub async fn find_active_membership<C: ConnectionTrait>(
    conn: &C,
    user_id: &str,
    organization_id: &str,
  ) -> Result<Option<organization_members::Model>, DbErr> {
    organization_members::Entity::find()
      .filter(
        Condition::all()
          .add(organization_members::Column::OrganizationId.eq(organization_id.to_string()))
          .add(organization_members::Column::UserId.eq(user_id.to_string()))
          .add(organization_members::Column::DeletedAt.is_null()),
      )
      .one(conn)
      .await
  }

  pub async fn list_active_memberships_by_user<C: ConnectionTrait>(
    conn: &C,
    user_id: &str,
  ) -> Result<Vec<organization_members::Model>, DbErr> {
    organization_members::Entity::find()
      .filter(
        Condition::all()
          .add(organization_members::Column::UserId.eq(user_id.to_string()))
          .add(organization_members::Column::DeletedAt.is_null()),
      )
      .all(conn)
      .await
  }

  pub async fn list_active_memberships_by_organization<C: ConnectionTrait>(
    conn: &C,
    organization_id: &str,
  ) -> Result<Vec<organization_members::Model>, DbErr> {
    organization_members::Entity::find()
      .filter(
        Condition::all()
          .add(organization_members::Column::OrganizationId.eq(organization_id.to_string()))
          .add(organization_members::Column::DeletedAt.is_null()),
      )
      .all(conn)
      .await
  }

  pub async fn find_first_active_membership_by_user<C: ConnectionTrait>(
    conn: &C,
    user_id: &str,
  ) -> Result<Option<organization_members::Model>, DbErr> {
    organization_members::Entity::find()
      .filter(
        Condition::all()
          .add(organization_members::Column::UserId.eq(user_id.to_string()))
          .add(organization_members::Column::DeletedAt.is_null()),
      )
      .order_by_asc(organization_members::Column::CreatedAt)
      .one(conn)
      .await
  }

  pub async fn exists_active_membership<C: ConnectionTrait>(
    conn: &C,
    organization_id: &str,
    user_id: &str,
  ) -> Result<bool, DbErr> {
    Ok(
      organization_members::Entity::find()
        .filter(
          Condition::all()
            .add(organization_members::Column::OrganizationId.eq(organization_id.to_string()))
            .add(organization_members::Column::UserId.eq(user_id.to_string()))
            .add(organization_members::Column::DeletedAt.is_null()),
        )
        .one(conn)
        .await?
        .is_some(),
    )
  }

  pub async fn insert_organization_membership<C: ConnectionTrait>(
    conn: &C,
    active: organization_members::ActiveModel,
  ) -> Result<organization_members::Model, DbErr> {
    active.insert(conn).await
  }

  pub async fn update_organization_membership<C: ConnectionTrait>(
    conn: &C,
    model: organization_members::Model,
    role: Option<organization_members::MemberRole>,
    deleted_at: Option<Option<DateTimeWithTimeZone>>,
  ) -> Result<organization_members::Model, DbErr> {
    let mut active = model.into_active_model();
    if let Some(role) = role {
      active.role = Set(role);
    }
    if let Some(deleted_at) = deleted_at {
      active.deleted_at = Set(deleted_at);
    }
    active.updated_at = Set(Utc::now().into());
    active.update(conn).await
  }
}
