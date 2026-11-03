//! Batched, typed source lookups used to recheck search result visibility.
use crate::CollaborationService;
use gity_core::AppError;
use gity_entities::{
  project_issue_assignees, project_issue_comments,
  project_issues::{self, Column as IssueColumn, Entity as IssueEntity},
};
use sea_orm::{
  ColumnTrait, Condition, EntityTrait, ExprTrait, QueryFilter, QuerySelect, QueryTrait,
  sea_query::Expr,
};
use std::collections::{HashMap, HashSet};

const QUERY_BATCH: usize = 300;
const MAX_CONFIDENTIAL_IDS: usize = 10_000;

impl CollaborationService {
  /// Return confidential issue primary IDs authored by or assigned to `user`
  /// within the supplied project scope. Oversized results require narrower scope.
  pub async fn search_confidential_ids(
    &self,
    user: i64,
    projects: &[i64],
  ) -> Result<Vec<i64>, AppError> {
    let project_ids = unique(projects.iter().copied());
    if project_ids.is_empty() {
      return Ok(Vec::new());
    }

    let mut confidential_ids = HashSet::new();
    for project_batch in project_ids.chunks(QUERY_BATCH) {
      let assigned = project_issue_assignees::Entity::find()
        .select_only()
        .column(project_issue_assignees::Column::ProjectIssueId)
        .filter(project_issue_assignees::Column::UserId.eq(user))
        .into_query();
      let rows = IssueEntity::find()
        .select_only()
        .column(IssueColumn::Id)
        .filter(IssueColumn::ProjectId.is_in(project_batch.iter().copied()))
        .filter(IssueColumn::Confidential.eq(1))
        .filter(
          Condition::any()
            .add(IssueColumn::AuthorUserId.eq(user))
            .add(Expr::col(IssueColumn::Id).in_subquery(assigned)),
        )
        .limit((MAX_CONFIDENTIAL_IDS + 1) as u64)
        .into_tuple::<i64>()
        .all(&self.repository.connection)
        .await?;
      confidential_ids.extend(rows);
      ensure_confidential_limit(confidential_ids.len())?;
    }
    let mut confidential_ids = confidential_ids.into_iter().collect::<Vec<_>>();
    confidential_ids.sort_unstable();
    Ok(confidential_ids)
  }

  /// Resolve issue primary IDs within one project using batched `IN` queries.
  pub async fn search_issues_by_ids(
    &self,
    project: i64,
    ids: &[i64],
  ) -> Result<Vec<project_issues::Model>, AppError> {
    let ids = unique(ids.iter().copied());
    let mut result = Vec::new();
    for batch in ids.chunks(QUERY_BATCH) {
      result.extend(
        IssueEntity::find()
          .filter(IssueColumn::ProjectId.eq(project))
          .filter(IssueColumn::Id.is_in(batch.iter().copied()))
          .all(&self.repository.connection)
          .await?,
      );
    }
    Ok(result)
  }

  /// Resolve activity/todo targets to their parent issues. Unsupported target
  /// kinds remain absent so callers can fail closed without guessing.
  pub async fn issue_targets(
    &self,
    targets: &[(i64, String, i64)],
  ) -> Result<HashMap<(i64, String, i64), project_issues::Model>, AppError> {
    let mut issue_targets = HashSet::new();
    let mut comment_targets = HashSet::new();
    for (project, kind, target_id) in targets {
      match kind.as_str() {
        "issue" => {
          issue_targets.insert((*project, kind.clone(), *target_id));
        }
        "issue_comment" => {
          comment_targets.insert((*project, kind.clone(), *target_id));
        }
        _ => {}
      }
    }

    let mut resolved = HashMap::new();
    let issue_ids = unique(issue_targets.iter().map(|(_, _, id)| *id));
    let issue_projects = unique(issue_targets.iter().map(|(project, _, _)| *project));
    for project_batch in issue_projects.chunks(QUERY_BATCH) {
      for id_batch in issue_ids.chunks(QUERY_BATCH) {
        for issue in IssueEntity::find()
          .filter(IssueColumn::ProjectId.is_in(project_batch.iter().copied()))
          .filter(IssueColumn::Id.is_in(id_batch.iter().copied()))
          .all(&self.repository.connection)
          .await?
        {
          let key = (issue.project_id, "issue".to_owned(), issue.id);
          if issue_targets.contains(&key) {
            resolved.insert(key, issue);
          }
        }
      }
    }

    let comment_ids = unique(comment_targets.iter().map(|(_, _, id)| *id));
    let mut comments = Vec::new();
    for batch in comment_ids.chunks(QUERY_BATCH) {
      comments.extend(
        project_issue_comments::Entity::find()
          .filter(project_issue_comments::Column::Id.is_in(batch.iter().copied()))
          .all(&self.repository.connection)
          .await?,
      );
    }
    let comments_by_id = comments
      .iter()
      .map(|comment| (comment.id, comment))
      .collect::<HashMap<_, _>>();
    let parents = unique(comments.iter().map(|comment| comment.project_issue_id));
    let expected_projects = unique(comment_targets.iter().map(|(project, _, _)| *project));
    let mut parent_issues = HashMap::new();
    for project_batch in expected_projects.chunks(QUERY_BATCH) {
      for parent_batch in parents.chunks(QUERY_BATCH) {
        if parent_batch.is_empty() {
          continue;
        }
        for issue in IssueEntity::find()
          .filter(IssueColumn::ProjectId.is_in(project_batch.iter().copied()))
          .filter(IssueColumn::Id.is_in(parent_batch.iter().copied()))
          .all(&self.repository.connection)
          .await?
        {
          parent_issues.insert(issue.id, issue);
        }
      }
    }
    for (project, kind, comment_id) in comment_targets {
      let Some(comment) = comments_by_id.get(&comment_id) else {
        continue;
      };
      let Some(issue) = parent_issues.get(&comment.project_issue_id) else {
        continue;
      };
      if issue.project_id == project {
        resolved.insert((project, kind, comment_id), issue.clone());
      }
    }
    Ok(resolved)
  }
}

fn unique(values: impl IntoIterator<Item = i64>) -> Vec<i64> {
  values
    .into_iter()
    .collect::<HashSet<_>>()
    .into_iter()
    .collect()
}

fn ensure_confidential_limit(count: usize) -> Result<(), AppError> {
  if count > MAX_CONFIDENTIAL_IDS {
    return Err(AppError::bad_request(
      "confidential issue scope exceeds 10000 results; narrow the project scope",
    ));
  }
  Ok(())
}
