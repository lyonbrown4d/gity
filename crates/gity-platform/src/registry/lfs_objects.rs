use super::lfs::actor_id;
use super::*;
use crate::openapi::{lfs_schemas as spec, schemas as api};
use axum::{
  body::Body,
  extract::{Path, Query, State},
  http::HeaderMap,
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
  s.repository_access
    .lfs_object(p, oid)
    .await?
    .map(gity_persistence::model_value)
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
  let missing_storage_key = if let Some(row) = s.repository_access.lfs_object(p, oid).await? {
    if s.storage.exists(&row.storage_key).await? {
      s.storage.delete(&key).await?;
      return Ok(gity_persistence::model_value(row));
    }
    Some(row.storage_key)
  } else {
    None
  };
  let result = s
    .repository_access
    .save_lfs_object(
      p,
      oid,
      i64::try_from(size).map_err(|_| AppError::bad_request("lfs object is too large"))?,
      key.clone(),
      missing_storage_key.as_deref(),
    )
    .await;
  match &result {
    Ok(row) if row.storage_key != key => {
      s.storage.delete(&key).await?;
    }
    Err(_) => {
      let _ = s.storage.delete(&key).await;
    }
    _ => {}
  }
  result.map(gity_persistence::model_value)
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
  let mut rows = s
    .repository_access
    .lfs_objects(p, cursor, (limit + 1) as u64)
    .await?
    .into_iter()
    .map(|row| {
      let mut value = gity_persistence::model_value(row);
      value.as_object_mut().unwrap().remove("storage_key");
      value
    })
    .collect::<Vec<_>>();
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
