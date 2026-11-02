use crate::repository::CollaborationRepository;
use gity_core::AppError;
use gity_entities::{
  project_issue_assignees as assignees, project_merge_request_approvals as approvals,
  project_merge_request_participants as participants,
};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, EntityTrait, ExprTrait, QueryFilter, QueryOrder, QuerySelect,
  QueryTrait, Set, TransactionTrait, sea_query::Expr,
};

impl CollaborationRepository {
  pub async fn issue_assignees(
    &self,
    project: i64,
    iid: i64,
  ) -> Result<Vec<assignees::Model>, AppError> {
    let issue = self.issue(project, iid).await?;
    Ok(
      assignees::Entity::find()
        .filter(assignees::Column::ProjectIssueId.eq(issue.id))
        .order_by_asc(assignees::Column::CreatedAt)
        .order_by_asc(assignees::Column::Id)
        .all(&self.connection)
        .await?,
    )
  }
  pub async fn assigned_issues(
    &self,
    project: i64,
    issue_ids: Vec<i64>,
  ) -> Result<Vec<assignees::Model>, AppError> {
    use gity_entities::project_issues as issues;
    let scoped = issues::Entity::find()
      .select_only()
      .column(issues::Column::Id)
      .filter(issues::Column::ProjectId.eq(project))
      .filter(issues::Column::Id.is_in(issue_ids))
      .into_query();
    Ok(
      assignees::Entity::find()
        .filter(Expr::col(assignees::Column::ProjectIssueId).in_subquery(scoped))
        .order_by_asc(assignees::Column::CreatedAt)
        .order_by_asc(assignees::Column::Id)
        .all(&self.connection)
        .await?,
    )
  }
  pub async fn replace_assignees(
    &self,
    project: i64,
    iid: i64,
    users: Vec<i64>,
  ) -> Result<(), AppError> {
    let issue = self.issue(project, iid).await?;
    self.validate_users(&users).await?;
    let tx = self.connection.begin().await?;
    assignees::Entity::delete_many()
      .filter(assignees::Column::ProjectIssueId.eq(issue.id))
      .exec(&tx)
      .await?;
    let now = chrono::Utc::now().naive_utc();
    for user in users {
      assignees::ActiveModel {
        id: Set(gity_core::new_id()),
        project_issue_id: Set(issue.id),
        user_id: Set(user),
        created_at: Set(now),
        updated_at: Set(now),
      }
      .insert(&tx)
      .await?;
    }
    tx.commit().await?;
    Ok(())
  }
  pub async fn merge_participants(
    &self,
    project: i64,
    iid: i64,
  ) -> Result<Vec<participants::Model>, AppError> {
    let mr = self.merge_request(project, iid).await?;
    Ok(
      participants::Entity::find()
        .filter(participants::Column::MergeRequestId.eq(mr.id))
        .order_by_asc(participants::Column::CreatedAt)
        .order_by_asc(participants::Column::Id)
        .all(&self.connection)
        .await?,
    )
  }
  pub async fn replace_participants(
    &self,
    project: i64,
    iid: i64,
    role: String,
    users: Vec<i64>,
  ) -> Result<(), AppError> {
    if !["reviewer", "assignee"].contains(&role.as_str()) {
      return Err(AppError::bad_request("invalid participant role"));
    }
    let mr = self.merge_request(project, iid).await?;
    self.validate_users(&users).await?;
    let tx = self.connection.begin().await?;
    participants::Entity::delete_many()
      .filter(participants::Column::MergeRequestId.eq(mr.id))
      .filter(participants::Column::Role.eq(role.clone()))
      .exec(&tx)
      .await?;
    let now = chrono::Utc::now().naive_utc();
    for user in users {
      participants::ActiveModel {
        id: Set(gity_core::new_id()),
        merge_request_id: Set(mr.id),
        user_id: Set(user),
        role: Set(role.clone()),
        created_at: Set(now),
        updated_at: Set(now),
      }
      .insert(&tx)
      .await?;
    }
    tx.commit().await?;
    Ok(())
  }
  pub async fn merge_approvals(
    &self,
    project: i64,
    iid: i64,
  ) -> Result<Vec<approvals::Model>, AppError> {
    let mr = self.merge_request(project, iid).await?;
    Ok(
      approvals::Entity::find()
        .filter(approvals::Column::MergeRequestId.eq(mr.id))
        .order_by_asc(approvals::Column::CreatedAt)
        .order_by_asc(approvals::Column::Id)
        .all(&self.connection)
        .await?,
    )
  }
  pub async fn approve(
    &self,
    project: i64,
    iid: i64,
    user: i64,
    approved: bool,
  ) -> Result<(), AppError> {
    let mr = self.merge_request(project, iid).await?;
    if mr.state != "opened" {
      return Err(AppError::conflict("merge request is not opened"));
    }
    let existing = approvals::Entity::find()
      .filter(approvals::Column::MergeRequestId.eq(mr.id))
      .filter(approvals::Column::UserId.eq(user))
      .one(&self.connection)
      .await?;
    if approved && existing.is_none() {
      let now = chrono::Utc::now().naive_utc();
      approvals::ActiveModel {
        id: Set(gity_core::new_id()),
        merge_request_id: Set(mr.id),
        user_id: Set(user),
        created_at: Set(now),
        updated_at: Set(now),
      }
      .insert(&self.connection)
      .await?;
    } else if !approved && let Some(row) = existing {
      approvals::Entity::delete_by_id(row.id)
        .exec(&self.connection)
        .await?;
    }
    Ok(())
  }
}
