use crate::BaseRepository;
use chrono::Utc;
use entity::repository_issue_comments;
use sea_orm::{
  ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder,
  QuerySelect,
};

#[derive(Clone)]
pub struct RepositoryIssueCommentsRepository<C: ConnectionTrait = DatabaseConnection> {
  base: BaseRepository<C>,
}

impl<C: ConnectionTrait> RepositoryIssueCommentsRepository<C> {
  pub fn new(conn: C) -> Self {
    Self {
      base: BaseRepository::new(conn),
    }
  }

  pub fn connection(&self) -> &C {
    self.base.connection()
  }

  pub async fn list_comments_by_issue(
    &self,
    issue_id: &str,
    limit: u64,
  ) -> Result<Vec<repository_issue_comments::Model>, DbErr> {
    repository_issue_comments::Entity::find()
      .filter(repository_issue_comments::Column::IssueId.eq(issue_id.to_string()))
      .order_by_asc(repository_issue_comments::Column::CreatedAt)
      .limit(limit)
      .all(self.connection())
      .await
  }

  pub async fn insert_comment(
    &self,
    active: repository_issue_comments::ActiveModel,
  ) -> Result<repository_issue_comments::Model, DbErr> {
    self.base.insert(active).await
  }

  pub async fn update_comment_content(
    &self,
    model: repository_issue_comments::Model,
    content: String,
  ) -> Result<repository_issue_comments::Model, DbErr> {
    let mut active: repository_issue_comments::ActiveModel = model.into();
    active.content = sea_orm::Set(content);
    active.updated_at = sea_orm::Set(Utc::now().into());
    self.base.update(active).await
  }
}
