use super::*;
#[derive(serde::Deserialize)]
struct FileCommitRequest {
  #[serde(flatten)]
  commit: gity_git::CreateFileCommitInput,
  #[serde(default)]
  skip_pipeline: bool,
}
pub(super) async fn write(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((id, action)): Path<(i64, String)>,
  Json(mut b): Json<Value>,
) -> Reply {
  let authorization =
    access::project_action(&s, &h, id, access::ProjectAction::RepositoryPush).await?;
  let actor = authorization.actor()?;
  let p = authorization.project;
  let path = repo(&p);

  match action.as_str() {
    "branches" => {
      let name = b["name"]
        .as_str()
        .or_else(|| b["branch_name"].as_str())
        .ok_or_else(|| AppError::bad_request("branch name required"))?;
      check_branch(&s, &h, &p, name, false, false).await?;
      let source = b["source_ref"]
        .as_str()
        .or_else(|| b["ref"].as_str())
        .unwrap_or(default_branch(&p));
      s.git
        .create_branch(&path, name, source)
        .await
        .map_err(git_error)?;
      let sha = s
        .git
        .resolve_revision(&path, name, default_branch(&p))
        .await
        .map_err(git_error)?;
      crate::events::repository_changed(&s, &p, name, &sha).await;
      Ok(ok(
        json!({"repository_id":id,"name":name,"last_commit_sha":sha,"is_protected":false,"protection":null}),
      ))
    }
    "files" | "file-commits" => {
      if b.get("file_path").is_none() {
        b["file_path"] = b["path"].clone();
      }
      if b["branch_name"].as_str().is_none_or(str::is_empty) {
        b["branch_name"] = json!(default_branch(&p));
      }
      let request: FileCommitRequest =
        serde_json::from_value(b).map_err(|e| AppError::bad_request(e.to_string()))?;
      let mut input = request.commit;
      check_branch(&s, &h, &p, &input.branch_name, false, false).await?;
      input.author_name = actor.username;
      let user = s.identity.user_model(actor.user_id).await?;
      input.author_email = user
        .and_then(|user| user.email)
        .unwrap_or_else(|| "noreply@gity.local".to_owned());
      let (branch, file, message) = (
        input.branch_name.clone(),
        input.file_path.clone(),
        input.message.clone(),
      );
      let sha = s
        .git
        .create_file_commit(&path, input)
        .await
        .map_err(git_error)?;
      crate::events::repository_changed_with_options(
        &s,
        &p,
        &branch,
        &sha,
        crate::events::RepositoryChangeOptions {
          skip_pipeline: request.skip_pipeline,
        },
      )
      .await;
      Ok(ok(
        json!({"repository_id":id,"branch_name":branch,"file_path":file,"path":file,"message":message,"commit_sha":sha}),
      ))
    }
    _ => Err(AppError::not_found("repository operation not found")),
  }
}
pub(super) async fn delete_branch(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((id, branch)): Path<(i64, String)>,
) -> Reply {
  let p = access::project_action(&s, &h, id, access::ProjectAction::RepositoryAdmin)
    .await?
    .project;
  if branch == default_branch(&p) {
    return Err(AppError::bad_request("cannot delete default branch"));
  }
  check_branch(&s, &h, &p, &branch, true, false).await?;
  s.git
    .delete_branch(&repo(&p), &branch)
    .await
    .map_err(git_error)?;
  let actor = auth::principal(&s, &h).await?;
  crate::events::project_change(
    &s,
    &p,
    actor.user_id,
    crate::events::ProjectChange::BranchDeleted(&branch),
  )
  .await?;
  Ok(ok(json!({"ok":true})))
}
