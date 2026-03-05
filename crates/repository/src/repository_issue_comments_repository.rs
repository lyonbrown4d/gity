use chrono::Utc;
use entity::repository_issue_comments;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter, QueryOrder,
  QuerySelect,
};

pub struct RepositoryIssueCommentsRepository;

impl RepositoryIssueCommentsRepository {
  pub async fn list_comments_by_issue<C: ConnectionTrait>(
    conn: &C,
    issue_id: &str,
    limit: u64,
  ) -> Result<Vec<repository_issue_comments::Model>, DbErr> {
    repository_issue_comments::Entity::find()
      .filter(repository_issue_comments::Column::IssueId.eq(issue_id.to_string()))
      .order_by_asc(repository_issue_comments::Column::CreatedAt)
      .limit(limit)
      .all(conn)
      .await
  }

  pub async fn insert_comment<C: ConnectionTrait>(
    conn: &C,
    active: repository_issue_comments::ActiveModel,
  ) -> Result<repository_issue_comments::Model, DbErr> {
    active.insert(conn).await
  }

  pub async fn update_comment_content<C: ConnectionTrait>(
    conn: &C,
    model: repository_issue_comments::Model,
    content: String,
  ) -> Result<repository_issue_comments::Model, DbErr> {
    let mut active: repository_issue_comments::ActiveModel = model.into();
    active.content = sea_orm::Set(content);
    active.updated_at = sea_orm::Set(Utc::now().into());
    active.update(conn).await
  }
}
