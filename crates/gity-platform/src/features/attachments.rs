use super::*;
use crate::openapi::{issue_schemas as spec, schemas as api};
use axum::{
  extract::{DefaultBodyLimit, Multipart, Path, Query, State},
  http::{HeaderMap, HeaderValue, header},
  response::{IntoResponse, Response},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use std::collections::HashMap;

pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  let shared = utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(list, create))
    .routes(utoipa_axum::routes!(detail))
    .routes(utoipa_axum::routes!(raw))
    .routes(utoipa_axum::routes!(draft))
    .routes(utoipa_axum::routes!(upload));
  let mut router = utoipa_axum::router::OpenApiRouter::new()
    .nest("/projects", shared.clone())
    .nest("/repos", shared);
  router = router.layer(DefaultBodyLimit::max(20 * 1024 * 1024));
  router
}
async fn load(s: &AppState, p: i64, i: i64, a: i64) -> Result<Value, AppError> {
  Ok(model_value(s.collaboration.attachment(p, i, a).await?))
}
#[utoipa::path(
    get,path="/{id}/issues/{iid}/attachments",tag="attachments",
    params(api::IidPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::Attachment>>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn list(State(s): State<AppState>, h: HeaderMap, Path((p, i)): Path<(i64, i64)>) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::Read).await?;
  response(json!(
    s.collaboration
      .attachments(p, i)
      .await?
      .into_iter()
      .map(model_value)
      .collect::<Vec<_>>()
  ))
}
async fn store(
  s: &AppState,
  p: i64,
  i: i64,
  u: i64,
  name: &str,
  ctype: &str,
  content: &[u8],
) -> Result<Value, AppError> {
  if name.trim().is_empty() || content.is_empty() {
    return Err(AppError::bad_request(
      "attachment file_name and content are required",
    ));
  }
  if content.len() > 15 * 1024 * 1024 {
    return Err(AppError::bad_request("attachment exceeds 15 MiB"));
  }
  let id = gity_core::new_id();
  let key = format!("issues/{p}/{id}");
  let issue = if i > 0 {
    Some(model_value(s.collaboration.issue(p, i).await?))
  } else {
    None
  };
  s.storage
    .put(&key, bytes::Bytes::copy_from_slice(content))
    .await?;
  let ctype = if ctype.trim().is_empty() {
    "application/octet-stream"
  } else {
    ctype.trim()
  };
  if let Some(issue) = issue {
    let result = s
      .collaboration
      .create_attachment(
        p,
        i,
        AttachmentCreate {
          id,
          project_issue_id: number(&issue, "id"),
          uploaded_by_user_id: u,
          file_name: name.trim().to_owned(),
          content_type: Some(ctype.to_owned()),
          byte_size: content.len() as i64,
          storage_key: key.clone(),
        },
      )
      .await
      .map(model_value);
    if result.is_err() {
      let _ = s.storage.delete(&key).await;
    }
    result
  } else {
    Ok(
      json!({"storage_key":key,"file_name":name.trim(),"content_type":ctype,"byte_size":content.len()}),
    )
  }
}
#[utoipa::path(
    post,path="/{id}/issues/{iid}/attachments",tag="attachments",
    params(api::IidPath),
    request_body(content=spec::CreateAttachment),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Attachment>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn create(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
  Json(b): Json<Value>,
) -> Reply {
  let actor = crate::access::project_action(&s, &h, p, ProjectAction::IssueCreate)
    .await?
    .actor()?;
  let content = STANDARD
    .decode(required(&b, "content_base64")?)
    .map_err(|_| AppError::bad_request("invalid content_base64"))?;
  response(
    store(
      &s,
      p,
      i,
      actor.user_id,
      &required(&b, "file_name")?,
      string(&b, "content_type"),
      &content,
    )
    .await?,
  )
}
#[utoipa::path(
    get,path="/{id}/issues/{iid}/attachments/{attachment}",tag="attachments",
    params(api::AttachmentPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::AttachmentContent>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn detail(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i, a)): Path<(i64, i64, i64)>,
) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::Read).await?;
  let row = load(&s, p, i, a).await?;
  let data = s.storage.get(string(&row, "storage_key")).await?;
  response(json!({"attachment":row,"content_base64":STANDARD.encode(data)}))
}
async fn bytes(s: &AppState, row: Value) -> Result<Response, AppError> {
  let data = s.storage.stream(string(&row, "storage_key")).await?;
  let mut response = axum::body::Body::from_stream(data).into_response();
  response.headers_mut().insert(
    header::CONTENT_TYPE,
    HeaderValue::from_str(string(&row, "content_type"))
      .unwrap_or(HeaderValue::from_static("application/octet-stream")),
  );
  response.headers_mut().insert(
    header::X_CONTENT_TYPE_OPTIONS,
    HeaderValue::from_static("nosniff"),
  );
  // Serve active documents as downloads, preventing uploads from executing in the application origin.
  response.headers_mut().insert(
    header::CONTENT_DISPOSITION,
    HeaderValue::from_static("attachment"),
  );
  Ok(response)
}
#[utoipa::path(
    get,path="/{id}/issues/{iid}/attachments/{attachment}/raw",tag="attachments",
    params(api::AttachmentPath),
    responses((status=200,description="Attachment bytes",body=api::Binary,content_type="application/octet-stream"),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn raw(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i, a)): Path<(i64, i64, i64)>,
) -> Result<Response, AppError> {
  crate::access::project_action(&s, &h, p, ProjectAction::Read).await?;
  let row = load(&s, p, i, a).await?;
  bytes(&s, row).await
}
#[utoipa::path(
    get,path="/{id}/issues/attachments/raw",tag="attachments",
    params(api::ProjectPath,("object_key"=String,Query,description="Project-scoped draft object key")),
    responses((status=200,description="Attachment bytes",body=api::Binary,content_type="application/octet-stream"),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn draft(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(p): Path<i64>,
  Query(q): Query<HashMap<String, String>>,
) -> Result<Response, AppError> {
  crate::access::project_action(&s, &h, p, ProjectAction::Read).await?;
  let key = q
    .get("object_key")
    .ok_or_else(|| AppError::bad_request("object_key is required"))?;
  if !key.starts_with(&format!("issues/{p}/")) {
    return Err(AppError::bad_request("invalid attachment object key"));
  }
  bytes(
    &s,
    json!({"storage_key":key,"content_type":"application/octet-stream"}),
  )
  .await
}
#[utoipa::path(
    post,path="/{id}/issues/attachments",tag="attachments",
    params(api::ProjectPath,("issue_id"=Option<i64>,Query,description="Issue IID; omit for a draft")),
    request_body(content=spec::MultipartAttachment,content_type="multipart/form-data"),
    responses((status=201,description="Attachment uploaded",body=spec::UploadedAttachment),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn upload(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(p): Path<i64>,
  Query(q): Query<HashMap<String, String>>,
  mut multipart: Multipart,
) -> Result<Response, AppError> {
  let actor = crate::access::project_action(&s, &h, p, ProjectAction::IssueCreate)
    .await?
    .actor()?;
  let i = q
    .get("issue_id")
    .map(|v| {
      v.parse::<i64>()
        .map_err(|_| AppError::bad_request("invalid issue_id"))
    })
    .transpose()?
    .unwrap_or(0);
  if i < 0 {
    return Err(AppError::bad_request("invalid issue_id"));
  }
  while let Some(field) = multipart
    .next_field()
    .await
    .map_err(|_| AppError::bad_request("invalid multipart body"))?
  {
    if field.name() == Some("file") {
      let name = field.file_name().unwrap_or("attachment").to_owned();
      let ctype = field
        .content_type()
        .unwrap_or("application/octet-stream")
        .to_owned();
      let data = field
        .bytes()
        .await
        .map_err(|_| AppError::bad_request("invalid file"))?;
      let row = store(&s, p, i, actor.user_id, &name, &ctype, &data).await?;
      let url = if i > 0 {
        format!(
          "/api/v1/projects/{p}/issues/{i}/attachments/{}/raw",
          number(&row, "id")
        )
      } else {
        format!(
          "/api/v1/projects/{p}/issues/attachments/raw?object_key={}",
          string(&row, "storage_key")
        )
      };
      let label = name.replace(']', "\\]");
      let prefix = if ctype.starts_with("image/") { "!" } else { "" };
      return Ok((axum::http::StatusCode::CREATED,Json(externalize(json!({"url":url,"object_key":row["storage_key"],"file_name":name,"content_type":ctype,"size":data.len(),"markdown":format!("{prefix}[{label}]({url})")})))).into_response());
    }
  }
  Err(AppError::bad_request("file is required"))
}
