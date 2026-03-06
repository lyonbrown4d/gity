use crate::BaseRepository;
use chrono::Utc;
use entity::repository_branches;
use sea_orm::{
  ColumnTrait, Condition, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, IntoActiveModel,
  QueryFilter, QueryOrder, Set, prelude::DateTimeWithTimeZone,
};

#[derive(Clone)]
pub struct RepositoryBranchesRepository<C: ConnectionTrait = DatabaseConnection> {
  base: BaseRepository<C>,
}

impl<C: ConnectionTrait> RepositoryBranchesRepository<C> {
  pub fn new(conn: C) -> Self {
    Self {
      base: BaseRepository::new(conn),
    }
  }

  pub fn connection(&self) -> &C {
    self.base.connection()
  }

  pub async fn list_repository_branches_by_repo_id(
    &self,
    repository_id: &str,
    include_deleted: bool,
  ) -> Result<Vec<repository_branches::Model>, DbErr> {
    let mut query = repository_branches::Entity::find()
      .filter(repository_branches::Column::RepositoryId.eq(repository_id.to_string()));

    if !include_deleted {
      query = query.filter(repository_branches::Column::DeletedAt.is_null());
    }

    query
      .order_by_asc(repository_branches::Column::Name)
      .all(self.connection())
      .await
  }

  pub async fn find_active_branch_by_repo_and_name(
    &self,
    repository_id: &str,
    branch_name: &str,
  ) -> Result<Option<repository_branches::Model>, DbErr> {
    repository_branches::Entity::find()
      .filter(
        Condition::all()
          .add(repository_branches::Column::RepositoryId.eq(repository_id.to_string()))
          .add(repository_branches::Column::Name.eq(branch_name.to_string()))
          .add(repository_branches::Column::DeletedAt.is_null()),
      )
      .one(self.connection())
      .await
  }

  pub async fn exists_active_branch_by_repo_and_name(
    &self,
    repository_id: &str,
    branch_name: &str,
  ) -> Result<bool, DbErr> {
    Ok(
      self
        .find_active_branch_by_repo_and_name(repository_id, branch_name)
        .await?
        .is_some(),
    )
  }

  pub async fn insert_branch(
    &self,
    active: repository_branches::ActiveModel,
  ) -> Result<repository_branches::Model, DbErr> {
    self.base.insert(active).await
  }

  pub async fn update_branch(
    &self,
    model: repository_branches::Model,
    is_protected: Option<bool>,
    last_commit_sha: Option<Option<String>>,
    deleted_at: Option<Option<DateTimeWithTimeZone>>,
  ) -> Result<repository_branches::Model, DbErr> {
    let mut active = model.into_active_model();
    if let Some(is_protected) = is_protected {
      active.is_protected = Set(is_protected);
    }
    if let Some(last_commit_sha) = last_commit_sha {
      active.last_commit_sha = Set(last_commit_sha);
    }
    if let Some(deleted_at) = deleted_at {
      active.deleted_at = Set(deleted_at);
    }
    active.updated_at = Set(Utc::now().into());
    self.base.update(active).await
  }
}
