//! Discussion use cases: listing threads, attaching comments and resolving.
use crate::{CollaborationService, domain::*};
use gity_core::AppError;
use gity_entities::{project_discussions, project_issue_comments, project_merge_request_comments};

impl CollaborationService {
  pub async fn issue_discussions(
    &self,
    project: i64,
    iid: i64,
  ) -> Result<
    Vec<(
      project_discussions::Model,
      Vec<project_issue_comments::Model>,
    )>,
    AppError,
  > {
    let target_id = self
      .discussion_target_id(project, DiscussionTarget::Issue, iid)
      .await?;
    let mut threads = Vec::new();
    for row in self
      .repository
      .discussions(project, DiscussionTarget::Issue.as_str(), target_id)
      .await?
    {
      let comments = self.repository.issue_discussion_comments(row.id).await?;
      threads.push((row, comments));
    }
    Ok(threads)
  }

  pub async fn merge_discussions(
    &self,
    project: i64,
    iid: i64,
  ) -> Result<
    Vec<(
      project_discussions::Model,
      Vec<project_merge_request_comments::Model>,
    )>,
    AppError,
  > {
    let target = DiscussionTarget::MergeRequest;
    let target_id = self.discussion_target_id(project, target, iid).await?;
    let mut threads = Vec::new();
    for row in self
      .repository
      .discussions(project, target.as_str(), target_id)
      .await?
    {
      let comments = self.repository.merge_discussion_comments(row.id).await?;
      threads.push((row, comments));
    }
    Ok(threads)
  }

  pub async fn issue_discussion(
    &self,
    project: i64,
    iid: i64,
    discussion: i64,
  ) -> Result<
    (
      project_discussions::Model,
      Vec<project_issue_comments::Model>,
    ),
    AppError,
  > {
    let target = DiscussionTarget::Issue;
    let target_id = self.discussion_target_id(project, target, iid).await?;
    let row = self
      .repository
      .discussion(project, target, target_id, discussion)
      .await?;
    let comments = self.repository.issue_discussion_comments(row.id).await?;
    Ok((row, comments))
  }

  pub async fn merge_discussion(
    &self,
    project: i64,
    iid: i64,
    discussion: i64,
  ) -> Result<
    (
      project_discussions::Model,
      Vec<project_merge_request_comments::Model>,
    ),
    AppError,
  > {
    let target = DiscussionTarget::MergeRequest;
    let target_id = self.discussion_target_id(project, target, iid).await?;
    let row = self
      .repository
      .discussion(project, target, target_id, discussion)
      .await?;
    let comments = self.repository.merge_discussion_comments(row.id).await?;
    Ok((row, comments))
  }

  /// Resolves or reopens a thread; the author or a project writer may do it.
  pub async fn set_discussion_resolved(
    &self,
    command: DiscussionResolve,
  ) -> Result<project_discussions::Model, AppError> {
    let target_id = self
      .discussion_target_id(command.project_id, command.target, command.iid)
      .await?;
    let row = self
      .repository
      .discussion(
        command.project_id,
        command.target,
        target_id,
        command.discussion_id,
      )
      .await?;
    let author = self
      .repository
      .discussion_author(command.target, row.id)
      .await?;
    if !command.can_write && author != Some(command.actor_user_id) {
      return Err(AppError::forbidden(
        "only the discussion author or a project writer may resolve it",
      ));
    }
    self
      .repository
      .set_discussion_resolved(row.id, command.resolved, command.actor_user_id)
      .await
  }

  /// Thread a new comment joins; a top-level comment opens its own thread.
  pub(crate) async fn comment_discussion(
    &self,
    project: i64,
    target: DiscussionTarget,
    target_id: i64,
    requested: Option<i64>,
  ) -> Result<i64, AppError> {
    match requested {
      Some(id) => Ok(
        self
          .repository
          .discussion(project, target, target_id, id)
          .await?
          .id,
      ),
      None => Ok(
        self
          .repository
          .create_discussion(DiscussionCreate {
            project_id: project,
            target_type: target.as_str().to_owned(),
            target_id,
          })
          .await?
          .id,
      ),
    }
  }

  /// Project-scoped primary key of the issue or merge request.
  async fn discussion_target_id(
    &self,
    project: i64,
    target: DiscussionTarget,
    iid: i64,
  ) -> Result<i64, AppError> {
    Ok(match target {
      DiscussionTarget::Issue => self.repository.issue(project, iid).await?.id,
      DiscussionTarget::MergeRequest => self.repository.merge_request(project, iid).await?.id,
    })
  }
}

/// Accepts a JSON object anchor and rejects anchors whose numbers are not numbers.
pub(crate) fn validate_position(raw: Option<String>) -> Result<Option<String>, AppError> {
  let Some(text) = raw else {
    return Ok(None);
  };
  let text = text.trim().to_owned();
  if text.is_empty() {
    return Ok(None);
  }
  let value: serde_json::Value = serde_json::from_str(&text)
    .map_err(|_| AppError::bad_request("position must be a JSON object"))?;
  let object = value
    .as_object()
    .ok_or_else(|| AppError::bad_request("position must be a JSON object"))?;
  if let Some(file) = object.get("file") {
    let file = file
      .as_str()
      .ok_or_else(|| AppError::bad_request("position.file must be a string"))?;
    if file.trim().is_empty() {
      return Err(AppError::bad_request("position.file must not be empty"));
    }
    for key in ["line", "column"] {
      if let Some(value) = object.get(key)
        && value.as_i64().is_none_or(|number| number <= 0)
      {
        return Err(AppError::bad_request(format!(
          "position.{key} must be a positive integer"
        )));
      }
    }
    if !object.contains_key("line") {
      return Err(AppError::bad_request("position.line is required"));
    }
  }
  Ok(Some(text))
}
