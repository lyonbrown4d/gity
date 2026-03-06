use crate::BaseRepository;
use chrono::Utc;
use entity::repositories;
use sea_orm::{
  ColumnTrait, Condition, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, IntoActiveModel,
  PaginatorTrait, QueryFilter, QueryOrder, Set, prelude::DateTimeWithTimeZone,
};

#[derive(Clone)]
pub struct RepositoriesRepository<C: ConnectionTrait = DatabaseConnection> {
  base: BaseRepository<C>,
}

impl<C: ConnectionTrait> RepositoriesRepository<C> {
  pub fn new(conn: C) -> Self {
    Self {
      base: BaseRepository::new(conn),
    }
  }

  pub fn connection(&self) -> &C {
    self.base.connection()
  }

  pub async fn find_active_repository_by_id(
    &self,
    repo_id: &str,
  ) -> Result<Option<repositories::Model>, DbErr> {
    let by_id = self
      .base
      .find_by_id::<repositories::Entity, _>(repo_id.to_string())
      .await?;
    if let Some(model) = by_id
      && model.deleted_at.is_none()
    {
      return Ok(Some(model));
    }

    repositories::Entity::find()
      .filter(
        Condition::all()
          .add(repositories::Column::Uuid.eq(repo_id.to_string()))
          .add(repositories::Column::DeletedAt.is_null()),
      )
      .one(self.connection())
      .await
  }

  pub async fn find_active_repository_by_org_and_key(
    &self,
    organization_id: &str,
    repo_key: &str,
  ) -> Result<Option<repositories::Model>, DbErr> {
    repositories::Entity::find()
      .filter(
        Condition::all()
          .add(repositories::Column::OrganizationId.eq(organization_id.to_string()))
          .add(repositories::Column::Key.eq(repo_key.to_string()))
          .add(repositories::Column::DeletedAt.is_null()),
      )
      .one(self.connection())
      .await
  }

  pub async fn exists_active_repository_by_org_and_key(
    &self,
    organization_id: &str,
    repo_key: &str,
  ) -> Result<bool, DbErr> {
    Ok(
      self
        .find_active_repository_by_org_and_key(organization_id, repo_key)
        .await?
        .is_some(),
    )
  }

  pub async fn list_active_repositories_by_org(
    &self,
    organization_id: &str,
  ) -> Result<Vec<repositories::Model>, DbErr> {
    repositories::Entity::find()
      .filter(
        Condition::all()
          .add(repositories::Column::OrganizationId.eq(organization_id.to_string()))
          .add(repositories::Column::DeletedAt.is_null()),
      )
      .all(self.connection())
      .await
  }

  pub async fn list_active_repositories(
    &self,
    organization_id: Option<&str>,
  ) -> Result<Vec<repositories::Model>, DbErr> {
    let mut query = repositories::Entity::find().filter(repositories::Column::DeletedAt.is_null());
    if let Some(organization_id) = organization_id {
      query = query.filter(repositories::Column::OrganizationId.eq(organization_id.to_string()));
    }

    query
      .order_by_desc(repositories::Column::CreatedAt)
      .all(self.connection())
      .await
  }

  pub async fn list_active_repositories_by_ids(
    &self,
    repository_ids: Vec<String>,
    organization_id: Option<&str>,
  ) -> Result<Vec<repositories::Model>, DbErr> {
    if repository_ids.is_empty() {
      return Ok(vec![]);
    }

    let mut condition = Condition::all()
      .add(repositories::Column::Id.is_in(repository_ids))
      .add(repositories::Column::DeletedAt.is_null());
    if let Some(organization_id) = organization_id {
      condition =
        condition.add(repositories::Column::OrganizationId.eq(organization_id.to_string()));
    }

    repositories::Entity::find()
      .filter(condition)
      .order_by_desc(repositories::Column::CreatedAt)
      .all(self.connection())
      .await
  }

  pub async fn list_active_repositories_paginated(
    &self,
    organization_id: Option<&str>,
    page: u64,
    page_size: u64,
  ) -> Result<(Vec<repositories::Model>, u64), DbErr> {
    let mut query = repositories::Entity::find().filter(repositories::Column::DeletedAt.is_null());
    if let Some(organization_id) = organization_id {
      query = query.filter(repositories::Column::OrganizationId.eq(organization_id.to_string()));
    }

    let paginator = query
      .order_by_desc(repositories::Column::CreatedAt)
      .paginate(self.connection(), page_size);
    let total = paginator.num_items().await?;
    let items = paginator.fetch_page(page.saturating_sub(1)).await?;
    Ok((items, total))
  }

  pub async fn insert_repository(
    &self,
    active: repositories::ActiveModel,
  ) -> Result<repositories::Model, DbErr> {
    self.base.insert(active).await
  }

  pub async fn update_repository(
    &self,
    model: repositories::Model,
    description: Option<Option<String>>,
    visibility: Option<repositories::RepositoryVisibility>,
    default_branch: Option<String>,
    deleted_at: Option<Option<DateTimeWithTimeZone>>,
  ) -> Result<repositories::Model, DbErr> {
    let mut active = model.into_active_model();
    if let Some(description) = description {
      active.description = Set(description);
    }
    if let Some(visibility) = visibility {
      active.visibility = Set(visibility);
    }
    if let Some(default_branch) = default_branch {
      active.default_branch = Set(default_branch);
    }
    if let Some(deleted_at) = deleted_at {
      active.deleted_at = Set(deleted_at);
    }
    active.updated_at = Set(Utc::now().into());
    self.base.update(active).await
  }
}
