use super::packages::{Upload, coordinate, named, upload};
use super::*;
use crate::openapi::{package_schemas as spec, schemas as api};
use axum::{
    body::Body,
    extract::{OriginalUri, Path, State},
    http::HeaderMap,
    response::{Html, IntoResponse, Response},
};
pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
    let shared =
        utoipa_axum::router::OpenApiRouter::new().routes(utoipa_axum::routes!(publish, download));
    utoipa_axum::router::OpenApiRouter::new()
        .routes(utoipa_axum::routes!(maven_publish, maven_download))
        .routes(utoipa_axum::routes!(
            super::package_metadata::npm,
            super::package_metadata::npm_publish
        ))
        .routes(utoipa_axum::routes!(super::package_metadata::nuget))
        .routes(utoipa_axum::routes!(pypi))
        .routes(utoipa_axum::routes!(pypi_package))
        .nest("/projects/{id}/packages/generic", shared.clone())
        .nest("/projects/{id}/packages/nuget", shared.clone())
        .nest("/projects/{id}/packages/pypi", shared)
        .layer(axum::middleware::from_fn(crate::auth::basic_challenge))
}
fn kind(uri: &axum::http::Uri) -> &str {
    uri.path()
        .split("/packages/")
        .nth(1)
        .and_then(|v| v.split('/').next())
        .unwrap_or("generic")
}
fn ctype(h: &HeaderMap) -> &str {
    h.get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
}
#[utoipa::path(
    put,path="/{name}/{version}/{*file}",tag="package protocols",
    params(api::ProtocolFilePath),
    request_body(content=api::Binary,content_type="application/octet-stream"),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::PackageFile>),api::Errors),
    security(("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn publish(
    State(s): State<AppState>,
    h: HeaderMap,
    Path((p, name, version, file)): Path<(i64, String, String, String)>,
    OriginalUri(uri): OriginalUri,
    body: Body,
) -> Reply {
    authorize(&s, &h, p, ProjectAction::PackageWrite).await?;
    reply(
        upload(
            &s,
            p,
            Upload {
                kind: kind(&uri),
                name: &name,
                version: &version,
                path: &file,
                content_type: ctype(&h),
            },
            body,
        )
        .await?,
    )
}
#[utoipa::path(
    get,path="/{name}/{version}/{*file}",tag="package protocols",
    params(api::ProtocolFilePath,("Range"=Option<String>,Header),("If-Range"=Option<String>,Header),("If-None-Match"=Option<String>,Header),("If-Match"=Option<String>,Header),("If-Modified-Since"=Option<String>,Header),("If-Unmodified-Since"=Option<String>,Header)),
    responses((status=200,description="Package bytes",body=api::Binary,content_type="application/octet-stream"),(status=206,description="Requested single byte range",body=api::Binary,content_type="application/octet-stream"),(status=304,description="Cached representation is current"),(status=412,description="Conditional request precondition failed"),(status=416,description="Byte range is unsatisfiable"),api::Errors),
    security((), ("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn download(
    State(s): State<AppState>,
    h: HeaderMap,
    Path((p, name, version, file)): Path<(i64, String, String, String)>,
    OriginalUri(uri): OriginalUri,
) -> Result<Response, AppError> {
    authorize(&s, &h, p, ProjectAction::PackageRead).await?;
    super::package_binary::response(
        &s,
        &h,
        coordinate(&s, p, kind(&uri), &name, &version, &file).await?,
    )
    .await
}
fn maven(file: &str) -> Result<(String, String), AppError> {
    let parts = file.split('/').collect::<Vec<_>>();
    if parts.len() < 3 {
        return Err(AppError::bad_request("maven package path is invalid"));
    }
    let artifact = parts[parts.len() - 3];
    let version = parts[parts.len() - 2];
    let group = parts[..parts.len() - 3].join(".");
    Ok((
        if group.is_empty() {
            artifact.to_owned()
        } else {
            format!("{group}:{artifact}")
        },
        version.into(),
    ))
}
#[utoipa::path(
    put,path="/projects/{id}/packages/maven/{*file}",tag="package protocols",
    params(api::MavenPath),
    request_body(content=api::Binary,content_type="application/octet-stream"),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::PackageFile>),api::Errors),
    security(("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn maven_publish(
    State(s): State<AppState>,
    h: HeaderMap,
    Path((p, file)): Path<(i64, String)>,
    body: Body,
) -> Reply {
    authorize(&s, &h, p, ProjectAction::PackageWrite).await?;
    let (name, version) = maven(&file)?;
    reply(
        upload(
            &s,
            p,
            Upload {
                kind: "maven",
                name: &name,
                version: &version,
                path: &file,
                content_type: ctype(&h),
            },
            body,
        )
        .await?,
    )
}
#[utoipa::path(
    get,path="/projects/{id}/packages/maven/{*file}",tag="package protocols",
    params(api::MavenPath,("Range"=Option<String>,Header),("If-Range"=Option<String>,Header),("If-None-Match"=Option<String>,Header),("If-Match"=Option<String>,Header),("If-Modified-Since"=Option<String>,Header),("If-Unmodified-Since"=Option<String>,Header)),
    responses((status=200,description="Package bytes",body=api::Binary,content_type="application/octet-stream"),(status=206,description="Requested single byte range",body=api::Binary,content_type="application/octet-stream"),(status=304,description="Cached representation is current"),(status=412,description="Conditional request precondition failed"),(status=416,description="Byte range is unsatisfiable"),api::Errors),
    security((), ("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn maven_download(
    State(s): State<AppState>,
    h: HeaderMap,
    Path((p, file)): Path<(i64, String)>,
) -> Result<Response, AppError> {
    authorize(&s, &h, p, ProjectAction::PackageRead).await?;
    let (name, version) = maven(&file)?;
    super::package_binary::response(
        &s,
        &h,
        coordinate(&s, p, "maven", &name, &version, &file).await?,
    )
    .await
}
fn escape(v: &str) -> String {
    v.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
#[utoipa::path(
    get,path="/projects/{id}/packages/pypi/simple",tag="package protocols",
    params(api::ProjectPath),
    responses((status=200,description="Python simple repository index",body=String,content_type="text/html"),api::Errors),
    security((), ("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn pypi(
    State(s): State<AppState>,
    h: HeaderMap,
    Path(p): Path<i64>,
) -> Result<Response, AppError> {
    authorize(&s, &h, p, ProjectAction::PackageRead).await?;
    let mut html = String::from("<!doctype html><html><body>\n");
    for row in
        s.db.all(
            "SELECT name FROM project_packages WHERE project_id = ? AND type = ? ORDER BY name",
            vec![p.into(), "pypi".into()],
        )
        .await?
    {
        let name = escape(text(&row, "name"));
        html.push_str(&format!("<a href=\"./{name}/\">{name}</a><br/>\n"));
    }
    html.push_str("</body></html>\n");
    Ok(Html(html).into_response())
}
#[utoipa::path(
    get,path="/projects/{id}/packages/pypi/simple/{name}",tag="package protocols",
    params(api::PackageNamePath),
    responses((status=200,description="Python simple repository index",body=String,content_type="text/html"),api::Errors),
    security((), ("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn pypi_package(
    State(s): State<AppState>,
    h: HeaderMap,
    Path((p, name)): Path<(i64, String)>,
) -> Result<Response, AppError> {
    authorize(&s, &h, p, ProjectAction::PackageRead).await?;
    let detail = named(&s, p, "pypi", &name).await?;
    let mut html = String::from("<!doctype html><html><body>\n");
    for version in detail["versions"].as_array().into_iter().flatten() {
        for file in version["files"].as_array().into_iter().flatten() {
            html.push_str(&format!(
                "<a href=\"/api/v1/projects/{p}/packages/files/{}/download\">{}</a><br/>\n",
                num(file, "id"),
                escape(text(file, "file_name"))
            ));
        }
    }
    html.push_str("</body></html>\n");
    Ok(Html(html).into_response())
}
