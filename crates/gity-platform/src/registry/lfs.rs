use super::lfs_objects::{object, upload, validate_oid};
use super::*;
use axum::{
  body::{Body, to_bytes},
  extract::Request,
  http::{HeaderMap, Method, StatusCode, header},
  response::{IntoResponse, Response},
};
use std::collections::HashMap;
pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(super::lfs_objects::objects))
    .routes(utoipa_axum::routes!(
      super::lfs_objects::locks,
      super::lfs_objects::create_lock
    ))
    .routes(utoipa_axum::routes!(super::lfs_objects::unlock))
}
pub fn is_lfs_path(path: &str) -> bool {
  path.contains(".git/info/lfs/")
}
pub async fn handle_lfs(s: AppState, request: Request) -> Result<Response, AppError> {
  let (parts, body) = request.into_parts();
  let (repo, tail) = parts
    .uri
    .path()
    .trim_start_matches('/')
    .split_once("/info/lfs/")
    .ok_or_else(|| AppError::not_found("lfs endpoint not found"))?;
  let project_path = repo
    .strip_suffix(".git")
    .ok_or_else(|| AppError::bad_request("invalid repository path"))?;
  let project = s
    .db
    .one(
      "SELECT * FROM projects WHERE full_path = ? AND deleted_at IS NULL",
      vec![project_path.into()],
    )
    .await?
    .ok_or_else(|| AppError::not_found("project not found"))?;
  let p = num(&project, "id");
  let h = &parts.headers;
  let q = parts
    .uri
    .query()
    .map(|q| {
      reqwest::Url::parse(&format!("http://local/?{q}"))
        .unwrap()
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect::<HashMap<_, _>>()
    })
    .unwrap_or_default();
  if tail.starts_with("objects/") && tail != "objects/batch" {
    let oid = tail.trim_start_matches("objects/");
    validate_oid(oid)?;
    if parts.method == Method::PUT {
      authorize(&s, h, p, ProjectAction::RepositoryPush).await?;
      upload(&s, p, oid, body).await?;
      return Ok(StatusCode::OK.into_response());
    }
    if parts.method == Method::GET || parts.method == Method::HEAD {
      authorize(&s, h, p, ProjectAction::RepositoryRead).await?;
      let row = object(&s, p, oid).await?;
      let stream = s.storage.stream(text(&row, "storage_key")).await?;
      return Ok(
        (
          [
            (header::CONTENT_TYPE, "application/octet-stream"),
            (header::CONTENT_LENGTH, &num(&row, "byte_size").to_string()),
          ],
          Body::from_stream(stream),
        )
          .into_response(),
      );
    }
  }
  if tail == "objects/batch" && parts.method == Method::POST {
    let b = json_body(body).await?;
    let operation = required(&b, "operation")?.to_lowercase();
    if !["upload", "download"].contains(&operation.as_str()) {
      return Err(AppError::bad_request("unsupported lfs operation"));
    }
    authorize(
      &s,
      h,
      p,
      if operation == "upload" {
        ProjectAction::RepositoryPush
      } else {
        ProjectAction::RepositoryRead
      },
    )
    .await?;
    let mut out = Vec::new();
    let requests = b["objects"]
      .as_array()
      .ok_or_else(|| AppError::bad_request("lfs objects are required"))?;
    if requests.len() > 1000 {
      return Err(AppError::bad_request("too many lfs objects"));
    }
    for item in requests {
      let oid = text(item, "oid").trim();
      let size = num(item, "size");
      let mut result = json!({"oid":oid,"size":size});
      if validate_oid(oid).is_err() || size < 0 {
        result["error"] = json!({"code":400,"message":"invalid lfs object oid or size"});
        out.push(result);
        continue;
      }
      let row = s
        .db
        .one(
          "SELECT * FROM project_lfs_objects WHERE project_id = ? AND oid = ?",
          vec![p.into(), oid.into()],
        )
        .await?;
      let exists = if let Some(row) = &row {
        s.storage.exists(text(row, "storage_key")).await?
      } else {
        false
      };
      let href = format!(
        "{}/{repo}/info/lfs/objects/{oid}",
        s.settings.base_url.trim_end_matches('/')
      );
      if operation == "upload" {
        if !exists {
          result["actions"] = json!({"upload":{"href":href}});
        }
      } else if exists {
        result["size"] = row.unwrap()["byte_size"].clone();
        result["actions"] = json!({"download":{"href":href}});
      } else {
        result["error"] = json!({"code":404,"message":"lfs object not found"});
      }
      out.push(result);
    }
    return Ok(lfs_json(json!({"transfer":"basic","objects":out})));
  }
  let write = parts.method == Method::POST;
  authorize(
    &s,
    h,
    p,
    if write {
      ProjectAction::RepositoryPush
    } else {
      ProjectAction::RepositoryRead
    },
  )
  .await?;
  if tail == "locks" && parts.method == Method::GET {
    return Ok(lfs_json(super::lfs_locks::list(&s, p, &q, None).await?));
  }
  if tail == "locks" && parts.method == Method::POST {
    let b = json_body(body).await?;
    let user = actor_id(&s, h).await?;
    return Ok(
      (
        StatusCode::CREATED,
        lfs_json(super::lfs_locks::create(&s, p, user, &b).await?),
      )
        .into_response(),
    );
  }
  if tail == "locks/verify" && parts.method == Method::POST {
    let b = json_body(body).await?;
    let mut q = q;
    for key in ["cursor", "limit"] {
      if let Some(v) = b.get(key) {
        q.insert(
          key.into(),
          v.as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| v.to_string()),
        );
      }
    }
    return Ok(lfs_json(
      super::lfs_locks::list(&s, p, &q, Some(actor_id(&s, h).await?)).await?,
    ));
  }
  if tail.starts_with("locks/") && tail.ends_with("/unlock") && parts.method == Method::POST {
    let id = tail
      .trim_start_matches("locks/")
      .trim_end_matches("/unlock")
      .parse()
      .map_err(|_| AppError::bad_request("invalid lock id"))?;
    return Ok(lfs_json(
      super::lfs_locks::remove(&s, p, actor_id(&s, h).await?, id, &json_body(body).await?).await?,
    ));
  }
  Err(AppError::not_found("lfs endpoint not found"))
}
fn lfs_json(v: Value) -> Response {
  (
    [(header::CONTENT_TYPE, "application/vnd.git-lfs+json")],
    Json(externalize(v)),
  )
    .into_response()
}
async fn json_body(body: Body) -> Result<Value, AppError> {
  serde_json::from_slice(
    &to_bytes(body, 1024 * 1024)
      .await
      .map_err(|_| AppError::bad_request("invalid lfs request body"))?,
  )
  .map_err(|_| AppError::bad_request("invalid lfs JSON"))
}
pub(super) async fn actor_id(s: &AppState, h: &HeaderMap) -> Result<i64, AppError> {
  if let Some(raw) = token_from_headers(h) {
    if let Some(token) = authenticate_token(s, &raw).await? {
      return Ok(token.created_by_user_id);
    }
    return Ok(crate::auth::principal_token(s, &raw).await?.user_id);
  }
  Err(AppError::unauthorized("authentication required"))
}
