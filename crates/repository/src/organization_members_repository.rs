use crate::BaseRepository;
use chrono::Utc;
use entity::organization_members;
use sea_orm::{
  ColumnTrait, Condition, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, IntoActiveModel,
  QueryFilter, QueryOrder, Set, prelude::DateTimeWithTimeZone,
};

#[derive(Clone)]
pub struct OrganizationMembersRepository<C: ConnectionTrait = DatabaseConnection> {
  base: BaseRepository<C>,
}

impl<C: ConnectionTrait> OrganizationMembersRepository<C> {
  pub fn new(conn: C) -> Self {
    Self {
      base: BaseRepository::new(conn),
    }
  }

  pub fn connection(&self) -> &C {
    self.base.connection()
  }

  pub async fn find_active_membership(
    &self,
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
      .one(self.connection())
      .await
  }

  pub async fn list_active_memberships_by_user(
    &self,
    user_id: &str,
  ) -> Result<Vec<organization_members::Model>, DbErr> {
    organization_members::Entity::find()
      .filter(
        Condition::all()
          .add(organization_members::Column::UserId.eq(user_id.to_string()))
          .add(organization_members::Column::DeletedAt.is_null()),
      )
      .all(self.connection())
      .await
  }

  pub async fn list_active_memberships_by_organization(
    &self,
    organization_id: &str,
  ) -> Result<Vec<organization_members::Model>, DbErr> {
    organization_members::Entity::find()
      .filter(
        Condition::all()
          .add(organization_members::Column::OrganizationId.eq(organization_id.to_string()))
          .add(organization_members::Column::DeletedAt.is_null()),
      )
      .all(self.connection())
      .await
  }

  pub async fn find_first_active_membership_by_user(
    &self,
    user_id: &str,
  ) -> Result<Option<organization_members::Model>, DbErr> {
    organization_members::Entity::find()
      .filter(
        Condition::all()
          .add(organization_members::Column::UserId.eq(user_id.to_string()))
          .add(organization_members::Column::DeletedAt.is_null()),
      )
      .order_by_asc(organization_members::Column::CreatedAt)
      .one(self.connection())
      .await
  }

  pub async fn exists_active_membership(
    &self,
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
        .one(self.connection())
        .await?
        .is_some(),
    )
  }

  pub async fn insert_organization_membership(
    &self,
    active: organization_members::ActiveModel,
  ) -> Result<organization_members::Model, DbErr> {
    self.base.insert(active).await
  }

  pub async fn update_organization_membership(
    &self,
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
    self.base.update(active).await
  }
}
