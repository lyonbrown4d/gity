use crate::BaseRepository;
use chrono::Utc;
use entity::organizations;
use sea_orm::{
  ColumnTrait, Condition, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, IntoActiveModel,
  PaginatorTrait, QueryFilter, QueryOrder, Set, prelude::DateTimeWithTimeZone,
};

#[derive(Clone)]
pub struct OrganizationsRepository<C: ConnectionTrait = DatabaseConnection> {
  base: BaseRepository<C>,
}

impl<C: ConnectionTrait> OrganizationsRepository<C> {
  pub fn new(conn: C) -> Self {
    Self {
      base: BaseRepository::new(conn),
    }
  }

  pub fn connection(&self) -> &C {
    self.base.connection()
  }

  pub async fn find_active_organization_by_id(
    &self,
    organization_id: &str,
  ) -> Result<Option<organizations::Model>, DbErr> {
    organizations::Entity::find_by_id(organization_id.to_string())
      .filter(organizations::Column::DeletedAt.is_null())
      .one(self.connection())
      .await
  }

  pub async fn find_active_organization_by_key(
    &self,
    key: &str,
  ) -> Result<Option<organizations::Model>, DbErr> {
    organizations::Entity::find()
      .filter(
        Condition::all()
          .add(organizations::Column::Key.eq(key.to_string()))
          .add(organizations::Column::DeletedAt.is_null()),
      )
      .one(self.connection())
      .await
  }

  pub async fn list_active_organizations_by_ids(
    &self,
    organization_ids: Vec<String>,
  ) -> Result<Vec<organizations::Model>, DbErr> {
    if organization_ids.is_empty() {
      return Ok(vec![]);
    }

    organizations::Entity::find()
      .filter(
        Condition::all()
          .add(organizations::Column::Id.is_in(organization_ids))
          .add(organizations::Column::DeletedAt.is_null()),
      )
      .all(self.connection())
      .await
  }

  pub async fn list_active_organizations(&self) -> Result<Vec<organizations::Model>, DbErr> {
    organizations::Entity::find()
      .filter(organizations::Column::DeletedAt.is_null())
      .order_by_desc(organizations::Column::CreatedAt)
      .all(self.connection())
      .await
  }

  pub async fn list_active_organizations_paginated(
    &self,
    page: u64,
    page_size: u64,
  ) -> Result<(Vec<organizations::Model>, u64), DbErr> {
    let query = organizations::Entity::find()
      .filter(organizations::Column::DeletedAt.is_null())
      .order_by_desc(organizations::Column::CreatedAt);

    let paginator = query.paginate(self.connection(), page_size);
    let total = paginator.num_items().await?;
    let items = paginator.fetch_page(page.saturating_sub(1)).await?;
    Ok((items, total))
  }

  pub async fn find_organization_by_key(
    &self,
    key: &str,
  ) -> Result<Option<organizations::Model>, DbErr> {
    organizations::Entity::find()
      .filter(organizations::Column::Key.eq(key.to_string()))
      .one(self.connection())
      .await
  }

  pub async fn insert_organization(
    &self,
    active: organizations::ActiveModel,
  ) -> Result<organizations::Model, DbErr> {
    self.base.insert(active).await
  }

  pub async fn update_organization(
    &self,
    model: organizations::Model,
    key: Option<String>,
    name: Option<String>,
    status: Option<organizations::OrgStatus>,
    deleted_at: Option<Option<DateTimeWithTimeZone>>,
  ) -> Result<organizations::Model, DbErr> {
    let mut active = model.into_active_model();
    if let Some(key) = key {
      active.key = Set(key);
    }
    if let Some(name) = name {
      active.name = Set(name);
    }
    if let Some(status) = status {
      active.status = Set(status);
    }
    if let Some(deleted_at) = deleted_at {
      active.deleted_at = Set(deleted_at);
    }
    active.updated_at = Set(Utc::now().into());
    self.base.update(active).await
  }
}
