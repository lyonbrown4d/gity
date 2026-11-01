use super::packages::{Upload, named, upload};
use super::*;
use crate::openapi::{package_schemas as spec, schemas as api};
use axum::{
    body::Body,
    extract::{Path, State},
    http::HeaderMap,
};
use base64::{Engine, engine::general_purpose::STANDARD};
fn npm_metadata(p: i64, name: &str, detail: &Value) -> Value {
    let mut versions = Map::new();
    let mut latest = String::new();
    for version in detail["versions"].as_array().into_iter().flatten() {
        let key = text(&version["version"], "version");
        if key > latest.as_str() {
            latest = key.to_owned();
        }
        let files = version["files"].as_array().cloned().unwrap_or_default();
        let file = files
            .iter()
            .find(|v| text(v, "file_name").ends_with(".tgz"))
            .or_else(|| files.first());
        let dist=file.map(|v|json!({"tarball":format!("/api/v1/projects/{p}/packages/files/{}/download",num(v,"id"))})).unwrap_or(json!({}));
        versions.insert(key.into(), json!({"name":name,"version":key,"dist":dist}));
    }
    json!({"name":name,"dist-tags":{"latest":latest},"versions":versions})
}
#[utoipa::path(
    get,path="/projects/{id}/packages/npm/{*name}",tag="package protocols",
    params(api::PackageNamePath),
    responses((status=200,description="Registry protocol response",body=spec::NpmMetadata),api::Errors),
    security((), ("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
pub(super) async fn npm(
    State(s): State<AppState>,
    h: HeaderMap,
    Path((p, name)): Path<(i64, String)>,
) -> Result<Json<Value>, AppError> {
    authorize(&s, &h, p, ProjectAction::PackageRead).await?;
    Ok(Json(npm_metadata(
        p,
        &name,
        &named(&s, p, "npm", &name).await?,
    )))
}
#[utoipa::path(
    put,path="/projects/{id}/packages/npm/{*name}",tag="package protocols",
    params(api::PackageNamePath),
    request_body(content=spec::NpmPublish),
    responses((status=200,description="Registry protocol response",body=spec::NpmMetadata),api::Errors),
    security(("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
pub(super) async fn npm_publish(
    State(s): State<AppState>,
    h: HeaderMap,
    Path((p, path)): Path<(i64, String)>,
    Json(b): Json<Value>,
) -> Result<Json<Value>, AppError> {
    authorize(&s, &h, p, ProjectAction::PackageWrite).await?;
    let name = if text(&b, "name").is_empty() {
        path
    } else {
        text(&b, "name").to_owned()
    };
    let version = if text(&b["dist-tags"], "latest").is_empty() {
        b["versions"]
            .as_object()
            .and_then(|v| v.keys().max())
            .cloned()
            .unwrap_or_default()
    } else {
        text(&b["dist-tags"], "latest").into()
    };
    if name.is_empty() || version.is_empty() {
        return Err(AppError::bad_request(
            "npm package name and version are required",
        ));
    }
    let attachments = b["_attachments"]
        .as_object()
        .filter(|v| !v.is_empty())
        .ok_or_else(|| AppError::bad_request("npm package attachment is required"))?;
    let mut decoded = Vec::new();
    for (file, attachment) in attachments {
        decoded.push((
            file,
            attachment,
            STANDARD
                .decode(text(attachment, "data"))
                .map_err(|_| AppError::bad_request("npm package attachment is invalid"))?,
        ));
    }
    let mut uploaded = Value::Null;
    for (file, attachment, data) in decoded {
        uploaded = upload(
            &s,
            p,
            Upload {
                kind: "npm",
                name: &name,
                version: &version,
                path: file,
                content_type: text(attachment, "content_type"),
            },
            Body::from(data),
        )
        .await?;
    }
    let mut metadata = npm_metadata(p, &name, &named(&s, p, "npm", &name).await?);
    metadata["uploaded_file_id"] = json!(num(&uploaded, "id").to_string());
    Ok(Json(metadata))
}
#[utoipa::path(
    get,path="/projects/{id}/packages/nuget/index.json",tag="package protocols",
    params(api::ProjectPath),
    responses((status=200,description="Registry protocol response",body=spec::NugetIndex),api::Errors),
    security((), ("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
pub(super) async fn nuget(
    State(s): State<AppState>,
    h: HeaderMap,
    Path(p): Path<i64>,
) -> Result<Json<Value>, AppError> {
    authorize(&s, &h, p, ProjectAction::PackageRead).await?;
    let base = format!("/api/v1/projects/{p}/packages/nuget");
    Ok(Json(
        json!({"version":"3.0.0","resources":[{"@id":base,"@type":"PackagePublish/2.0.0"},{"@id":format!("{base}/download"),"@type":"PackageBaseAddress/3.0.0"}]}),
    ))
}
