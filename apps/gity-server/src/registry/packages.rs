use super::*;
use crate::openapi::{package_schemas as spec, schemas as api};
use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Path, State},
    http::HeaderMap,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use futures_util::StreamExt;
pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
    let shared = utoipa_axum::router::OpenApiRouter::new()
        .routes(utoipa_axum::routes!(list))
        .routes(utoipa_axum::routes!(detail))
        .routes(utoipa_axum::routes!(upload_json))
        .routes(utoipa_axum::routes!(content))
        .routes(utoipa_axum::routes!(download));
    utoipa_axum::router::OpenApiRouter::new()
        .nest("/projects", shared.clone())
        .nest("/repos", shared)
        .merge(super::package_protocols::documented_routes())
        .layer(DefaultBodyLimit::max(20 * 1024 * 1024))
}
#[utoipa::path(
    get,path="/{id}/packages",tag="packages",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::Package>>),api::Errors),
    security((), ("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn list(State(s): State<AppState>, h: HeaderMap, Path(p): Path<i64>) -> Reply {
    authorize(&s, &h, p, ProjectAction::PackageRead).await?;
    reply(json!(
        s.db.all(
            "SELECT * FROM project_packages WHERE project_id = ? ORDER BY id DESC",
            vec![p.into()]
        )
        .await?
    ))
}
#[utoipa::path(
    get,path="/{id}/packages/{package}",tag="packages",
    params(api::PackagePath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::PackageDetail>),api::Errors),
    security((), ("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn detail(State(s): State<AppState>, h: HeaderMap, Path((p, id)): Path<(i64, i64)>) -> Reply {
    authorize(&s, &h, p, ProjectAction::PackageRead).await?;
    reply(package(&s, p, id).await?)
}
pub(super) async fn package(s: &AppState, p: i64, id: i64) -> Result<Value, AppError> {
    let pkg = scoped(s, "project_packages", p, id).await?;
    let mut versions = Vec::new();
    for row in
        s.db.all(
            "SELECT * FROM project_package_versions WHERE project_package_id = ? ORDER BY id DESC",
            vec![id.into()],
        )
        .await?
    {
        let files=s.db.all("SELECT * FROM project_package_files WHERE project_package_version_id = ? ORDER BY id DESC",vec![num(&row,"id").into()]).await?;
        versions.push(json!({"version":row,"files":files}));
    }
    Ok(json!({"package":pkg,"versions":versions}))
}
pub(super) async fn named(s: &AppState, p: i64, kind: &str, name: &str) -> Result<Value, AppError> {
    let row =
        s.db.one(
            "SELECT id FROM project_packages WHERE project_id = ? AND type = ? AND name = ?",
            vec![p.into(), kind.into(), name.into()],
        )
        .await?
        .ok_or_else(|| AppError::not_found("package not found"))?;
    package(s, p, num(&row, "id")).await
}
pub(super) async fn file(s: &AppState, p: i64, id: i64) -> Result<Value, AppError> {
    let sql = concat!(
        "SELECT f.* FROM project_package_files f ",
        "JOIN project_package_versions v ON v.id=f.project_package_version_id ",
        "JOIN project_packages p ON p.id=v.project_package_id ",
        "WHERE p.project_id = ? AND f.id = ?"
    );
    s.db.one(sql, vec![p.into(), id.into()])
        .await?
        .ok_or_else(|| AppError::not_found("package file not found"))
}
pub(super) async fn coordinate(
    s: &AppState,
    p: i64,
    kind: &str,
    name: &str,
    version: &str,
    path: &str,
) -> Result<Value, AppError> {
    let basename = path.rsplit('/').next().unwrap_or(path);
    let sql = concat!(
        "SELECT f.* FROM project_package_files f ",
        "JOIN project_package_versions v ON v.id=f.project_package_version_id ",
        "JOIN project_packages p ON p.id=v.project_package_id ",
        "WHERE p.project_id = ? AND p.type = ? AND p.name = ? AND v.version = ? ",
        "AND (f.file_path = ? OR f.file_name = ?) ORDER BY f.id DESC LIMIT 1"
    );
    let args = vec![
        p.into(),
        kind.into(),
        name.into(),
        version.into(),
        path.into(),
        basename.into(),
    ];
    s.db.one(sql, args)
        .await?
        .ok_or_else(|| AppError::not_found("package file not found"))
}
pub(super) struct Upload<'a> {
    pub kind: &'a str,
    pub name: &'a str,
    pub version: &'a str,
    pub path: &'a str,
    pub content_type: &'a str,
}
pub(super) async fn upload(
    s: &AppState,
    p: i64,
    input: Upload<'_>,
    body: Body,
) -> Result<Value, AppError> {
    if [input.kind, input.name, input.version, input.path]
        .iter()
        .any(|v| v.trim().is_empty())
    {
        return Err(AppError::bad_request(
            "type, name, version, and file_name are required",
        ));
    }
    let path = input.path.trim_matches('/').replace('\\', "/");
    let name = path.rsplit('/').next().unwrap_or(&path);
    if name.is_empty() {
        return Err(AppError::bad_request("file_name is required"));
    }
    let id = gity_core::new_id();
    let key = format!("packages/{p}/{id}");
    let size = s
        .storage
        .put_stream(
            &key,
            body.into_data_stream()
                .map(|v| v.map_err(|_| AppError::bad_request("read package request body failed"))),
            1024 * 1024 * 1024,
        )
        .await?;
    let result = async {
        let package_fields = map(json!({
            "project_id": p,
            "type": input.kind,
            "name": input.name,
        }));
        let package_sql = "SELECT * FROM project_packages WHERE project_id = ? AND type = ? AND name = ?";
        let package_args = vec![p.into(), input.kind.into(), input.name.into()];
        let package = ensure(s, "project_packages", package_sql, package_args, package_fields).await?;
        let version_fields = map(json!({
            "project_package_id": num(&package,"id"),
            "version": input.version,
            "status": "default",
        }));
        let version_sql = "SELECT * FROM project_package_versions WHERE project_package_id = ? AND version = ?";
        let version_args = vec![num(&package,"id").into(), input.version.into()];
        let version = ensure(s, "project_package_versions", version_sql, version_args, version_fields).await?;
        let fields = map(json!({
            "id": id,
            "project_package_version_id": num(&version,"id"),
            "file_name": name,
            "file_path": path,
            "content_type": if input.content_type.is_empty() {"application/octet-stream"} else {input.content_type},
            "byte_size": size,
            "storage_key": key,
        }));
        s.db.insert("project_package_files", fields).await
    }.await;
    if result.is_err() {
        let _ = s.storage.delete(&key).await;
    }
    result
}
async fn ensure(
    s: &AppState,
    table: &str,
    sql: &str,
    args: Vec<sea_orm::Value>,
    fields: Map<String, Value>,
) -> Result<Value, AppError> {
    if let Some(row) = s.db.one(sql, args.clone()).await? {
        return Ok(row);
    }
    match s.db.insert(table, fields).await {
        Ok(row) => Ok(row),
        Err(error) if error.status == axum::http::StatusCode::CONFLICT => {
            s.db.one(sql, args).await?.ok_or(error)
        }
        Err(error) => Err(error),
    }
}
#[utoipa::path(
    post,path="/{id}/packages/files",tag="packages",
    params(api::ProjectPath),
    request_body(content=spec::CreatePackageFile),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::PackageFile>),api::Errors),
    security(("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn upload_json(
    State(s): State<AppState>,
    h: HeaderMap,
    Path(p): Path<i64>,
    Json(b): Json<Value>,
) -> Reply {
    authorize(&s, &h, p, ProjectAction::PackageWrite).await?;
    let data = STANDARD
        .decode(required(&b, "content_base64")?)
        .map_err(|_| AppError::bad_request("invalid content_base64"))?;
    let path = if text(&b, "file_path").is_empty() {
        required(&b, "file_name")?
    } else {
        text(&b, "file_path").to_owned()
    };
    reply(
        upload(
            &s,
            p,
            Upload {
                kind: &required(&b, "type")?,
                name: &required(&b, "name")?,
                version: &required(&b, "version")?,
                path: &path,
                content_type: text(&b, "content_type"),
            },
            Body::from(data),
        )
        .await?,
    )
}
#[utoipa::path(
    get,path="/{id}/packages/files/{file}",tag="packages",
    params(api::FilePath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::PackageContent>),api::Errors),
    security((), ("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn content(
    State(s): State<AppState>,
    h: HeaderMap,
    Path((p, id)): Path<(i64, i64)>,
) -> Reply {
    authorize(&s, &h, p, ProjectAction::PackageRead).await?;
    let row = file(&s, p, id).await?;
    let content = s.storage.get(text(&row, "storage_key")).await?;
    reply(json!({"file":row,"content_base64":STANDARD.encode(content)}))
}
#[utoipa::path(
    get,path="/{id}/packages/files/{file}/download",tag="packages",
    params(api::FilePath,("Range"=Option<String>,Header),("If-Range"=Option<String>,Header),("If-None-Match"=Option<String>,Header),("If-Match"=Option<String>,Header),("If-Modified-Since"=Option<String>,Header),("If-Unmodified-Since"=Option<String>,Header)),
    responses((status=200,description="Package bytes",body=api::Binary,content_type="application/octet-stream"),(status=206,description="Requested single byte range",body=api::Binary,content_type="application/octet-stream"),(status=304,description="Cached representation is current"),(status=412,description="Conditional request precondition failed"),(status=416,description="Byte range is unsatisfiable"),api::Errors),
    security((), ("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn download(
    State(s): State<AppState>,
    h: HeaderMap,
    Path((p, id)): Path<(i64, i64)>,
) -> Result<axum::response::Response, AppError> {
    authorize(&s, &h, p, ProjectAction::PackageRead).await?;
    super::package_binary::response(&s, &h, file(&s, p, id).await?).await
}
