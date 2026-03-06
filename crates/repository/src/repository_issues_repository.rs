use crate::BaseRepository;
use chrono::Utc;
use entity::repository_issues;
use sea_orm::{
  ColumnTrait, Condition, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, IntoActiveModel,
  QueryFilter, QueryOrder, QuerySelect, Set,
};

#[derive(Clone)]
pub struct RepositoryIssuesRepository<C: ConnectionTrait = DatabaseConnection> {
  base: BaseRepository<C>,
}

impl<C: ConnectionTrait> RepositoryIssuesRepository<C> {
  pub fn new(conn: C) -> Self {
    Self {
      base: BaseRepository::new(conn),
    }
  }

  pub fn connection(&self) -> &C {
    self.base.connection()
  }

  pub async fn list_issues_by_repo(
    &self,
    repository_id: &str,
    status: Option<repository_issues::RepositoryIssueStatus>,
    limit: u64,
  ) -> Result<Vec<repository_issues::Model>, DbErr> {
    let mut query = repository_issues::Entity::find()
      .filter(repository_issues::Column::RepositoryId.eq(repository_id.to_string()));
    if let Some(status) = status {
      query = query.filter(repository_issues::Column::Status.eq(status));
    }

    query
      .order_by_desc(repository_issues::Column::Number)
      .limit(limit)
      .all(self.connection())
      .await
  }

  pub async fn find_issue_by_repo_and_id(
    &self,
    repository_id: &str,
    issue_id: &str,
  ) -> Result<Option<repository_issues::Model>, DbErr> {
    repository_issues::Entity::find()
      .filter(
        Condition::all()
          .add(repository_issues::Column::RepositoryId.eq(repository_id.to_string()))
          .add(repository_issues::Column::Id.eq(issue_id.to_string())),
      )
      .one(self.connection())
      .await
  }

  pub async fn find_issue_by_repo_and_number(
    &self,
    repository_id: &str,
    number: i32,
  ) -> Result<Option<repository_issues::Model>, DbErr> {
    repository_issues::Entity::find()
      .filter(
        Condition::all()
          .add(repository_issues::Column::RepositoryId.eq(repository_id.to_string()))
          .add(repository_issues::Column::Number.eq(number)),
      )
      .one(self.connection())
      .await
  }

  pub async fn next_issue_number_by_repo(&self, repository_id: &str) -> Result<i32, DbErr> {
    let latest = repository_issues::Entity::find()
      .filter(repository_issues::Column::RepositoryId.eq(repository_id.to_string()))
      .order_by_desc(repository_issues::Column::Number)
      .one(self.connection())
      .await?;

    Ok(latest.map(|issue| issue.number + 1).unwrap_or(1))
  }

  pub async fn insert_issue(
    &self,
    active: repository_issues::ActiveModel,
  ) -> Result<repository_issues::Model, DbErr> {
    self.base.insert(active).await
  }

  pub async fn update_issue(
    &self,
    model: repository_issues::Model,
    title: Option<String>,
    description: Option<Option<String>>,
    status: Option<repository_issues::RepositoryIssueStatus>,
    assignee_user_id: Option<Option<String>>,
    closed_at: Option<Option<sea_orm::prelude::DateTimeWithTimeZone>>,
  ) -> Result<repository_issues::Model, DbErr> {
    let mut active = model.into_active_model();
    if let Some(title) = title {
      active.title = Set(title);
    }
    if let Some(description) = description {
      active.description = Set(description);
    }
    if let Some(status) = status {
      active.status = Set(status);
    }
    if let Some(assignee_user_id) = assignee_user_id {
      active.assignee_user_id = Set(assignee_user_id);
    }
    if let Some(closed_at) = closed_at {
      active.closed_at = Set(closed_at);
    }
    active.updated_at = Set(Utc::now().into());
    self.base.update(active).await
  }
}
