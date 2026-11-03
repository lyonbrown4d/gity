use super::*;
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    post,path="/projects",tag="projects",
    description="Requires developer or higher organization membership; initializes the bare Git repository.",
    request_body=spec::CreateProject,
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Project>),api::Errors),
    security(("bearer"=[]))
)]
pub(super) async fn create(State(s): State<AppState>, h: HeaderMap, Json(b): Json<Value>) -> Reply {
  let actor = auth::principal(&s, &h).await?;
  let name = required(&b, "name")?;
  let path_key = slug(
    b["key"]
      .as_str()
      .or_else(|| b["path_key"].as_str())
      .or_else(|| b["path"].as_str())
      .unwrap_or(name),
  )?;
  let branch = b["default_branch"].as_str().unwrap_or("main");
  s.git
    .validate_initial_branch(branch)
    .map_err(|_| AppError::bad_request("invalid default branch"))?;
  let organization_id = id(&b["organization_id"])?;
  let description = description(&b)?;
  let options = super::initialization::Initialization::parse(&b)?;
  let (organization, _) = s.projects.organization(&actor, organization_id, 30).await?;
  let files = options.files(name, description.as_deref(), &organization.name)?;
  let has_initial_commit = !files.is_empty();
  let initial_commit = has_initial_commit.then(|| gity_git::InitialCommit {
    files,
    author_name: actor.username.clone(),
    author_email: format!("{}@users.noreply.gity.local", actor.user_id),
  });
  let model = s
    .projects
    .create(
      &actor,
      gity_projects::CreateProject {
        organization_id,
        name: name.into(),
        path_key,
        description,
        visibility: visibility(&b)?.into(),
        default_branch: branch.into(),
      },
    )
    .await?;
  let project_id = model.id;
  let row = access::project_record(model);
  if let Err(error) = s
    .git
    .init_bare_with_commit(&crate::repository::repo(&row), branch, initial_commit)
    .await
  {
    s.projects
      .compensate_creation(project_id, actor.user_id)
      .await?;
    tracing::warn!(project_id,error=%error,"project creation compensated after Git initialization failed");
    return Err(AppError::new(
      axum::http::StatusCode::SERVICE_UNAVAILABLE,
      "Git storage is unavailable; project creation was not completed",
    ));
  }
  crate::events::project_change(
    &s,
    &row,
    actor.user_id,
    crate::events::ProjectChange::Created,
  )
  .await?;
  if has_initial_commit {
    // Initial files contain no pipeline configuration. Update search projections
    // without scheduling a pipeline solely because the repository was created.
    match s
      .git
      .resolve_revision(&crate::repository::repo(&row), branch, "")
      .await
    {
      Ok(sha) => {
        crate::events::repository_changed_with_options(
          &s,
          &row,
          &format!("refs/heads/{branch}"),
          &sha,
          crate::events::RepositoryChangeOptions {
            skip_pipeline: true,
          },
        )
        .await
      }
      Err(error) => {
        tracing::warn!(project_id,error=%error,"initial commit indexing deferred to reconciliation")
      }
    }
  }
  Ok(ok(project_view(&s, row)))
}
