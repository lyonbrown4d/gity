use super::lfs::actor_id;
use super::*;
use crate::openapi::{lfs_schemas as spec, schemas as api};
use axum::{
  body::Body,
  extract::{Path, Query, State},
  http::{HeaderMap, StatusCode},
};
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
pub(super) fn validate_oid(oid: &str) -> Result<(), AppError> {
  if oid.len() != 64 || !oid.bytes().all(|c| c.is_ascii_hexdigit()) {
    Err(AppError::bad_request(
      "lfs oid must be a 64-character hex string",
    ))
  } else {
    Ok(())
  }
}
pub(super) async fn object(s: &AppState, p: i64, oid: &str) -> Result<Value, AppError> {
  s.db
    .one(
      "SELECT * FROM project_lfs_objects WHERE project_id = ? AND oid = ?",
      vec![p.into(), oid.into()],
    )
    .await?
    .ok_or_else(|| AppError::not_found("lfs object not found"))
}
pub(super) async fn upload(s: &AppState, p: i64, oid: &str, body: Body) -> Result<Value, AppError> {
  let key = format!("lfs/{p}/{}", gity_core::new_id());
  let mut hash = Sha256::new();
  let stream = body.into_data_stream().map(|data| {
    data
      .inspect(|data| {
        hash.update(data);
      })
      .map_err(|_| AppError::bad_request("read lfs object failed"))
  });
  let size = s
    .storage
    .put_stream(&key, stream, 10 * 1024 * 1024 * 1024)
    .await?;
  if !hex::encode(hash.finalize()).eq_ignore_ascii_case(oid) {
    s.storage.delete(&key).await?;
    return Err(AppError::bad_request(
      "lfs object SHA256 does not match oid",
    ));
  }
  let row = s
    .db
    .one(
      "SELECT * FROM project_lfs_objects WHERE project_id = ? AND oid = ?",
      vec![p.into(), oid.into()],
    )
    .await?;
  if let Some(row) = row {
    if s.storage.exists(text(&row, "storage_key")).await? {
      s.storage.delete(&key).await?;
      return Ok(row);
    }
    return s
      .db
      .update(
        "project_lfs_objects",
        num(&row, "id"),
        map(json!({"byte_size":size,"storage_key":key})),
      )
      .await;
  }
  let result = s
    .db
    .insert(
      "project_lfs_objects",
      map(json!({"project_id":p,"oid":oid,"byte_size":size,"storage_key":key})),
    )
    .await;
  if let Err(e) = &result {
    let _ = s.storage.delete(&key).await;
    if e.status == StatusCode::CONFLICT {
      return object(s, p, oid).await;
    }
  }
  result
}
#[utoipa::path(
    get,path="/projects/{id}/lfs/objects",tag="lfs objects",
    params(api::ProjectPath,("cursor"=Option<String>,Query),("limit"=Option<i64>,Query)),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::LfsObjects>),api::Errors),
    security((), ("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
pub(super) async fn objects(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(p): Path<i64>,
  Query(q): Query<HashMap<String, String>>,
) -> Reply {
  authorize(&s, &h, p, ProjectAction::RepositoryRead).await?;
  let (cursor, limit) = super::lfs_locks::pagination(&q)?;
  let mut rows=s.db.all("SELECT id,project_id,oid,byte_size,created_at,updated_at FROM project_lfs_objects WHERE project_id = ? AND id > ? ORDER BY id LIMIT ?",vec![p.into(),cursor.into(),(limit+1).into()]).await?;
  let next = if rows.len() > limit as usize {
    rows.truncate(limit as usize);
    num(rows.last().unwrap(), "id").to_string()
  } else {
    String::new()
  };
  reply(json!({"objects":rows,"next_cursor":next}))
}
#[utoipa::path(
    get,path="/projects/{id}/lfs/locks",tag="lfs objects",
    params(api::ProjectPath,("path"=Option<String>,Query),("id"=Option<String>,Query),("cursor"=Option<String>,Query),("limit"=Option<i64>,Query)),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::LfsLocks>),api::Errors),
    security((), ("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
pub(super) async fn locks(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(p): Path<i64>,
  Query(q): Query<HashMap<String, String>>,
) -> Reply {
  authorize(&s, &h, p, ProjectAction::RepositoryRead).await?;
  reply(super::lfs_locks::list(&s, p, &q, None).await?)
}
#[utoipa::path(
    post,path="/projects/{id}/lfs/locks",tag="lfs objects",
    params(api::ProjectPath),
    request_body(content=spec::CreateLock),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::LockEnvelope>),api::Errors),
    security(("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
pub(super) async fn create_lock(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(p): Path<i64>,
  Json(b): Json<Value>,
) -> Reply {
  authorize(&s, &h, p, ProjectAction::RepositoryPush).await?;
  reply(super::lfs_locks::create(&s, p, actor_id(&s, &h).await?, &b).await?)
}
#[utoipa::path(
    post,path="/projects/{id}/lfs/locks/{lock}/unlock",tag="lfs objects",
    params(api::LockPath),
    request_body(content=spec::Unlock),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::LockEnvelope>),api::Errors),
    security(("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
pub(super) async fn unlock(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, id)): Path<(i64, i64)>,
  Json(b): Json<Value>,
) -> Reply {
  authorize(&s, &h, p, ProjectAction::RepositoryPush).await?;
  reply(super::lfs_locks::remove(&s, p, actor_id(&s, &h).await?, id, &b).await?)
}
