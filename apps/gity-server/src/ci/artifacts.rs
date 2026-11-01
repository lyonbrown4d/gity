use super::*;
use super::{job_schemas as jobs_spec, params, runner_schemas as runners_spec};
use crate::access;
use crate::openapi::schemas as api;
use axum::{
    extract::{Path, State},
    http::HeaderMap,
};
use base64::{Engine, engine::general_purpose::STANDARD};

#[utoipa::path(
    post, path="/runners/jobs/{job}/artifacts", tag="ci-runner-agent",
    summary="Store a runner artifact up to 64 MiB",
    params(params::AgentJobPath),
    request_body=runners_spec::UploadArtifact,
    responses((status=200,description="Store a runner artifact up to 64 MiB",body=api::Envelope<jobs_spec::Artifact>),api::Errors),
    security(())
)]
pub(crate) async fn upload(
    State(s): State<AppState>,
    Path(j): Path<i64>,
    Json(b): Json<Value>,
) -> Reply {
    let (_, job) = runners::owned(&s, &b, j).await?;
    let name = text(&b, "file_name");
    if !config::safe_relative(name) || name.contains('/') {
        return Err(AppError::bad_request(
            "artifact file_name must be a single safe filename",
        ));
    }
    let encoded = text(&b, "content_base64");
    if encoded.len() > 90 * 1024 * 1024 {
        return Err(AppError::bad_request("artifact exceeds 64 MiB"));
    }
    let bytes = STANDARD
        .decode(encoded)
        .map_err(|_| AppError::bad_request("invalid artifact base64"))?;
    if bytes.len() > 64 * 1024 * 1024 {
        return Err(AppError::bad_request("artifact exceeds 64 MiB"));
    }
    let p = job["project_id"].as_i64().unwrap();
    let artifact = new_id();
    let key = format!("ci/{p}/{j}/{artifact}");
    s.storage
        .put(&key, bytes::Bytes::from(bytes.clone()))
        .await?;
    let row =
        s.db.insert(
            "project_job_artifacts",
            json!({
                    "id":artifact,
                    "project_id":p,
                    "project_job_id":j,
                    "name":text(&b,
                        "name"),
                    "file_name":name,
                    "file_path":text(&b,
                        "file_path"),
                    "content_type":text(&b,
                        "content_type"),
                    "byte_size":bytes.len() as i64,
                    "storage_key":key
                }
            )
            .as_object()
            .unwrap()
            .clone(),
        )
        .await;
    match row {
        Ok(mut row) => {
            row.as_object_mut().unwrap().remove("storage_key");
            Ok(gity_core::ok(row))
        }
        Err(error) => {
            let _ = s.storage.delete(&key).await;
            Err(error)
        }
    }
}

#[utoipa::path(
    get, path="/{project}/jobs/{job}/artifacts", tag="ci",
    summary="List stored artifacts for a project job",
    params(params::JobPath),
    responses((status=200,description="List stored artifacts for a project job",body=api::Envelope<Vec<jobs_spec::Artifact>>),api::Errors),
    security(("bearer"=[]),("private_token"=[]))
)]
pub(crate) async fn list(
    State(s): State<AppState>,
    h: HeaderMap,
    Path((p, j)): Path<(i64, i64)>,
) -> Reply {
    access::project_action(&s, &h, p, ProjectAction::JobRead).await?;
    one(&s, "project_jobs", p, j).await?;
    let mut rows=s.db.all("SELECT * FROM project_job_artifacts WHERE project_id=? AND project_job_id=? ORDER BY id",vec![p.into(),j.into()]).await?;
    for row in &mut rows {
        row.as_object_mut().unwrap().remove("storage_key");
    }
    Ok(gity_core::ok(rows))
}
#[utoipa::path(
    get, path="/{project}/jobs/{job}/artifacts/{artifact}", tag="ci",
    summary="Download a stored project artifact as base64",
    params(params::ArtifactPath),
    responses((status=200,description="Download a stored project artifact as base64",body=api::Envelope<jobs_spec::ArtifactDownload>),api::Errors),
    security(("bearer"=[]),("private_token"=[]))
)]
pub(crate) async fn download(
    State(s): State<AppState>,
    h: HeaderMap,
    Path((p, j, a)): Path<(i64, i64, i64)>,
) -> Reply {
    access::project_action(&s, &h, p, ProjectAction::JobRead).await?;
    one(&s, "project_jobs", p, j).await?;
    let mut row =
        s.db.one(
            "SELECT * FROM project_job_artifacts WHERE project_id=? AND project_job_id=? AND id=?",
            vec![p.into(), j.into(), a.into()],
        )
        .await?
        .ok_or_else(|| AppError::not_found("artifact not found"))?;
    let key = text(&row, "storage_key");
    if !config::safe_relative(key) {
        return Err(AppError::internal("invalid artifact storage key"));
    }
    let bytes = s.storage.get(key).await?;
    row.as_object_mut().unwrap().remove("storage_key");
    row["encoding"] = json!("base64");
    row["content_base64"] = json!(STANDARD.encode(bytes));
    Ok(gity_core::ok(row))
}
