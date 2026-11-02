use super::views::pipeline_detail;
use super::*;
use serde::Deserialize;
#[derive(Default, Deserialize, utoipa::ToSchema)]
#[serde(default)]
pub struct CreatePipelineInput {
  pub source: String,
  pub ref_name: String,
  pub commit_sha: String,
  pub config_source: String,
  pub config_content: String,
}
pub async fn create_push_pipeline(
  s: &AppState,
  project_id: i64,
  ref_name: &str,
  commit_sha: &str,
) -> Result<Option<Value>, AppError> {
  if !commit_sha.is_empty() && commit_sha.bytes().all(|b| b == b'0') {
    return Ok(None);
  }
  if !ref_name.starts_with("refs/heads/") && ref_name.starts_with("refs/") {
    return Ok(None);
  }
  let branch = ref_name.strip_prefix("refs/heads/").unwrap_or(ref_name);
  let project = model_value(s.ci.project(project_id).await?);
  let repo = crate::repository::repo(&project);
  if !s
    .git
    .file_exists(
      &repo,
      commit_sha,
      text(&project, "default_branch"),
      ".gity-ci.yml",
    )
    .await
    .map_err(git_error)?
  {
    return Ok(None);
  }
  if let Some(existing) = s.ci.push_pipeline(project_id, branch, commit_sha).await? {
    return Ok(Some(pipeline_detail(s, project_id, existing.id).await?));
  }
  Ok(Some(
    create_pipeline(
      s,
      &project,
      CreatePipelineInput {
        source: "push".into(),
        ref_name: branch.into(),
        commit_sha: commit_sha.into(),
        ..Default::default()
      },
    )
    .await?,
  ))
}

pub async fn create_pipeline(
  s: &AppState,
  project: &Value,
  mut input: CreatePipelineInput,
) -> Result<Value, AppError> {
  if input.config_source.ends_with(".plano") {
    return Err(AppError::bad_request("CI supports .gity-ci.yml YAML only"));
  }
  let project_id = id(project);
  let repo = crate::repository::repo(project);
  if input.ref_name.is_empty() {
    input.ref_name = text(project, "default_branch").into();
  }
  if input.commit_sha.is_empty() {
    input.commit_sha = s
      .git
      .resolve_revision(&repo, &input.ref_name, text(project, "default_branch"))
      .await
      .map_err(git_error)?;
  } else {
    input.commit_sha = s
      .git
      .resolve_revision(&repo, &input.commit_sha, "")
      .await
      .map_err(git_error)?;
  }
  let mut trusted_config = false;
  if input.config_content.is_empty() {
    let blob = s
      .git
      .get_blob_limited(&repo, &input.commit_sha, "", ".gity-ci.yml", 1024 * 1024)
      .await
      .map_err(git_error)?;
    if blob["encoding"] != "utf-8" {
      return Err(AppError::bad_request("CI YAML must use UTF-8"));
    }
    input.config_content = text(&blob, "content").into();
    input.config_source = ".gity-ci.yml".into();
    trusted_config = true;
  } else if s
    .git
    .file_exists(&repo, &input.commit_sha, "", ".gity-ci.yml")
    .await
    .map_err(git_error)?
  {
    let pinned = s
      .git
      .get_blob_limited(&repo, &input.commit_sha, "", ".gity-ci.yml", 1024 * 1024)
      .await
      .map_err(git_error)?;
    trusted_config =
      pinned["encoding"] == "utf-8" && text(&pinned, "content") == input.config_content;
  }
  if input.config_source.is_empty() {
    input.config_source = "inline".into();
  }
  if input.source.is_empty() {
    input.source = "web".into();
  }
  let spec = config::parse(&input.config_content)?;
  let mut jobs = Vec::with_capacity(spec.jobs.len());
  for (name, job) in &spec.jobs {
    let mut env = spec.variables.clone();
    env.extend(job.variables.clone());
    let needs = job.needs.clone().unwrap_or_default();
    let payload = json!({
      "pipeline_name":name,"project_full_path":text(project,"full_path"),
      "ref_name":input.ref_name,"commit_sha":input.commit_sha,"stage":job.stage,
      "image":job.image,"needs":needs,"script":job.script.lines(),
      "artifacts":job.artifacts.paths(),"tags":job.tags,"env":env,"yaml_env":env,
      "trusted_config":trusted_config,"timeout_seconds":job.timeout_seconds,
      "config_source":input.config_source,"pipeline_job_name":name
    });
    jobs.push(gity_ci::PipelineJobSpec {
      name: name.clone(),
      stage: job.stage.clone(),
      image: job.image.clone(),
      needs,
      script: job.script.lines(),
      artifacts: job.artifacts.paths(),
      payload,
    });
  }
  let pipeline_id = s
    .ci
    .create_pipeline(gity_ci::NewPipeline {
      project_id,
      name: spec.name,
      source: input.source,
      ref_name: input.ref_name,
      commit_sha: input.commit_sha,
      config_source: input.config_source,
      config_content: input.config_content,
      jobs,
    })
    .await?;
  pipeline_detail(s, project_id, pipeline_id).await
}
