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
    let project =
        s.db.one(
            "SELECT * FROM projects WHERE id=? AND deleted_at IS NULL",
            vec![project_id.into()],
        )
        .await?
        .ok_or_else(|| AppError::not_found("project not found"))?;
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
    if let Some(existing)=s.db.one("SELECT id FROM project_pipelines WHERE project_id=? AND source='push' AND ref_name=? AND commit_sha=?",
        vec![project_id.into(),
            branch.into(),
            commit_sha.into()]).await?{
        return Ok(Some(pipeline_detail(s,
                    project_id,
                    id(&existing)).await?));

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
    let pipeline_id = new_id();
    let iid = s.db.next_iid(project_id, "pipeline").await?;
    let name = if spec.name.is_empty() {
        format!("Pipeline #{iid}")
    } else {
        spec.name.clone()
    };
    let tx = s.db.transaction().await?;
    if input.source == "push" {
        tx.execute_raw(s.db.statement(
            "UPDATE projects SET updated_at=updated_at WHERE id=?",
            vec![project_id.into()],
        ))
        .await?;
        let existing=tx_rows(s,
            &tx,
            "SELECT id FROM project_pipelines WHERE project_id=? AND source='push' AND ref_name=? AND commit_sha=?",
            vec![project_id.into(),
                input.ref_name.clone().into(),
                input.commit_sha.clone().into()]).await?;
        if let Some(row) = existing.first() {
            let existing_id = id(row);
            tx.commit().await?;
            return pipeline_detail(s, project_id, existing_id).await;
        }
    }
    insert_tx(
        s,
        &tx,
        "project_pipelines",
        json!({
                "id":pipeline_id,
                "project_id":project_id,
                "iid":iid,
                "name":name,
                "source":input.source,
                "ref_name":input.ref_name,
                "commit_sha":input.commit_sha,
                "status":"pending",
                "config_source":input.config_source,
                "config_content":input.config_content,
                "started_at":worker::ZERO,
                "finished_at":worker::ZERO
            }
        ),
    )
    .await?;
    for (order, (name, job)) in spec.jobs.iter().enumerate() {
        let mut env = spec.variables.clone();
        env.extend(job.variables.clone());
        let needs = job.needs.clone().unwrap_or_default();
        let payload = json!({
                "pipeline_id":pipeline_id,
                "pipeline_iid":iid,
                "pipeline_name":name,
                "project_full_path":text(project,
                    "full_path"),
                "ref_name":input.ref_name,
                "commit_sha":input.commit_sha,
                "stage":job.stage,
                "image":job.image,
                "needs":needs,
                "script":job.script.lines(),
                "artifacts":job.artifacts.paths(),
                "tags":job.tags,
                "env":env,
                "yaml_env":env,
                "trusted_config":trusted_config,
                "timeout_seconds":job.timeout_seconds,
                "config_source":input.config_source,
                "pipeline_job_name":name
            }
        );
        let j = insert_tx(
            s,
            &tx,
            "project_jobs",
            json!({
                    "project_id":project_id,
                    "kind":"script",
                    "status":"pending",
                    "payload":payload.to_string(),
                    "result":"",
                    "attempts":0,
                    "max_attempts":3,
                    "run_after":if needs.is_empty(){
                        now()
                    }
                    else{
                        worker::BLOCKED.into()
                    }
                    ,
                    "locked_by":"",
                    "locked_until":worker::ZERO,
                    "last_error":"",
                    "started_at":worker::ZERO,
                    "finished_at":worker::ZERO
                }
            ),
        )
        .await?;
        insert_tx(
            s,
            &tx,
            "project_pipeline_jobs",
            json!({
                    "project_id":project_id,
                    "pipeline_id":pipeline_id,
                    "project_job_id":id(&j),
                    "name":name,
                    "stage":job.stage,
                    "needs":serde_json::to_string(&needs).unwrap(),
                    "image":job.image,
                    "script":serde_json::to_string(&job.script.lines()).unwrap(),
                    "artifacts":serde_json::to_string(&job.artifacts.paths()).unwrap(),
                    "sort_order":order as i64
                }
            ),
        )
        .await?;
    }
    tx.commit().await?;
    pipeline_detail(s, project_id, pipeline_id).await
}
