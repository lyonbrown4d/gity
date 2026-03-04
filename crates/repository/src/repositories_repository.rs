use chrono::Utc;
use entity::repositories;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DbErr, EntityTrait, IntoActiveModel,
  QueryFilter, QueryOrder, Set, prelude::DateTimeWithTimeZone,
};

pub struct RepositoriesRepository;

impl RepositoriesRepository {
  pub async fn find_active_repository_by_id<C: ConnectionTrait>(
    conn: &C,
    repo_id: &str,
  ) -> Result<Option<repositories::Model>, DbErr> {
    repositories::Entity::find_by_id(repo_id.to_string())
      .filter(repositories::Column::DeletedAt.is_null())
      .one(conn)
      .await
  }

  pub async fn find_active_repository_by_org_and_key<C: ConnectionTrait>(
    conn: &C,
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
      .one(conn)
      .await
  }

  pub async fn exists_active_repository_by_org_and_key<C: ConnectionTrait>(
    conn: &C,
    organization_id: &str,
    repo_key: &str,
  ) -> Result<bool, DbErr> {
    Ok(
      Self::find_active_repository_by_org_and_key(conn, organization_id, repo_key)
        .await?
        .is_some(),
    )
  }

  pub async fn list_active_repositories_by_org<C: ConnectionTrait>(
    conn: &C,
    organization_id: &str,
  ) -> Result<Vec<repositories::Model>, DbErr> {
    repositories::Entity::find()
      .filter(
        Condition::all()
          .add(repositories::Column::OrganizationId.eq(organization_id.to_string()))
          .add(repositories::Column::DeletedAt.is_null()),
      )
      .all(conn)
      .await
  }

  pub async fn list_active_repositories<C: ConnectionTrait>(
    conn: &C,
    organization_id: Option<&str>,
  ) -> Result<Vec<repositories::Model>, DbErr> {
    let mut query = repositories::Entity::find().filter(repositories::Column::DeletedAt.is_null());
    if let Some(organization_id) = organization_id {
      query = query.filter(repositories::Column::OrganizationId.eq(organization_id.to_string()));
    }

    query
      .order_by_desc(repositories::Column::CreatedAt)
      .all(conn)
      .await
  }

  pub async fn insert_repository<C: ConnectionTrait>(
    conn: &C,
    active: repositories::ActiveModel,
  ) -> Result<repositories::Model, DbErr> {
    active.insert(conn).await
  }

  pub async fn update_repository<C: ConnectionTrait>(
    conn: &C,
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
    active.update(conn).await
  }
}
