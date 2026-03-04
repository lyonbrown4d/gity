use chrono::Utc;
use entity::repository_branches;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DbErr, EntityTrait, IntoActiveModel,
  QueryFilter, QueryOrder, Set, prelude::DateTimeWithTimeZone,
};

pub struct RepositoryBranchesRepository;

impl RepositoryBranchesRepository {
  pub async fn list_repository_branches_by_repo_id<C: ConnectionTrait>(
    conn: &C,
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
      .all(conn)
      .await
  }

  pub async fn find_active_branch_by_repo_and_name<C: ConnectionTrait>(
    conn: &C,
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
      .one(conn)
      .await
  }

  pub async fn exists_active_branch_by_repo_and_name<C: ConnectionTrait>(
    conn: &C,
    repository_id: &str,
    branch_name: &str,
  ) -> Result<bool, DbErr> {
    Ok(
      Self::find_active_branch_by_repo_and_name(conn, repository_id, branch_name)
        .await?
        .is_some(),
    )
  }

  pub async fn insert_branch<C: ConnectionTrait>(
    conn: &C,
    active: repository_branches::ActiveModel,
  ) -> Result<repository_branches::Model, DbErr> {
    active.insert(conn).await
  }

  pub async fn update_branch<C: ConnectionTrait>(
    conn: &C,
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
    active.update(conn).await
  }
}
