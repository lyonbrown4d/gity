use gity_core::{AppError, Db};
use gity_entities::{project_access_tokens, project_deploy_keys};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde_json::{Value, json};

pub(super) async fn list_tokens(
  db: &Db,
  project_id: i64,
  kind: &str,
) -> Result<Vec<project_access_tokens::Model>, AppError> {
  Ok(
    project_access_tokens::Entity::find()
      .filter(project_access_tokens::Column::ProjectId.eq(project_id))
      .filter(project_access_tokens::Column::Kind.eq(kind))
      .order_by_desc(project_access_tokens::Column::Id)
      .all(&db.connection)
      .await?,
  )
}

pub(super) async fn token_by_hash(
  db: &Db,
  digest: &str,
) -> Result<Option<project_access_tokens::Model>, AppError> {
  Ok(
    project_access_tokens::Entity::find()
      .filter(project_access_tokens::Column::Token.eq(digest))
      .one(&db.connection)
      .await?,
  )
}

pub(super) async fn scoped_token(
  db: &Db,
  project_id: i64,
  id: i64,
) -> Result<Option<project_access_tokens::Model>, AppError> {
  Ok(
    project_access_tokens::Entity::find()
      .filter(project_access_tokens::Column::ProjectId.eq(project_id))
      .filter(project_access_tokens::Column::Id.eq(id))
      .one(&db.connection)
      .await?,
  )
}

pub(super) struct NewProjectToken {
  pub(super) project_id: i64,
  pub(super) kind: String,
  pub(super) name: String,
  pub(super) username: String,
  pub(super) token_digest: String,
  pub(super) scopes: String,
  pub(super) created_by_user_id: i64,
  pub(super) expires_at: Option<chrono::NaiveDateTime>,
}

pub(super) async fn create_token(
  db: &Db,
  input: NewProjectToken,
) -> Result<project_access_tokens::Model, AppError> {
  let now = chrono::Utc::now().naive_utc();
  Ok(
    project_access_tokens::ActiveModel {
      id: Set(gity_core::new_id()),
      project_id: Set(input.project_id),
      kind: Set(input.kind),
      name: Set(input.name),
      username: Set(input.username),
      token: Set(input.token_digest),
      scopes: Set(input.scopes),
      created_by_user_id: Set(input.created_by_user_id),
      expires_at: Set(input.expires_at),
      revoked_at: Set(None),
      last_used_at: Set(None),
      created_at: Set(now),
      updated_at: Set(now),
    }
    .insert(&db.connection)
    .await?,
  )
}

pub(super) async fn mark_token_used(
  db: &Db,
  token: project_access_tokens::Model,
) -> Result<(), AppError> {
  let now = chrono::Utc::now().naive_utc();
  project_access_tokens::ActiveModel {
    id: Set(token.id),
    last_used_at: Set(Some(now)),
    updated_at: Set(now),
    ..Default::default()
  }
  .update(&db.connection)
  .await?;
  Ok(())
}

pub(super) async fn revoke_token(
  db: &Db,
  token: project_access_tokens::Model,
) -> Result<(), AppError> {
  let now = chrono::Utc::now().naive_utc();
  project_access_tokens::ActiveModel {
    id: Set(token.id),
    revoked_at: Set(Some(now)),
    updated_at: Set(now),
    ..Default::default()
  }
  .update(&db.connection)
  .await?;
  Ok(())
}

pub(super) async fn list_deploy_keys(
  db: &Db,
  project_id: i64,
) -> Result<Vec<project_deploy_keys::Model>, AppError> {
  Ok(
    project_deploy_keys::Entity::find()
      .filter(project_deploy_keys::Column::ProjectId.eq(project_id))
      .order_by_desc(project_deploy_keys::Column::Id)
      .all(&db.connection)
      .await?,
  )
}

pub(super) async fn scoped_deploy_key(
  db: &Db,
  project_id: i64,
  id: i64,
) -> Result<Option<project_deploy_keys::Model>, AppError> {
  Ok(
    project_deploy_keys::Entity::find()
      .filter(project_deploy_keys::Column::ProjectId.eq(project_id))
      .filter(project_deploy_keys::Column::Id.eq(id))
      .one(&db.connection)
      .await?,
  )
}

pub(super) async fn create_deploy_key(
  db: &Db,
  project_id: i64,
  title: String,
  public_key: String,
  fingerprint: String,
  can_push: bool,
  created_by_user_id: i64,
) -> Result<project_deploy_keys::Model, AppError> {
  let tx = db.transaction().await?;
  let now = chrono::Utc::now().naive_utc();
  let model = project_deploy_keys::ActiveModel {
    id: Set(gity_core::new_id()),
    project_id: Set(project_id),
    title: Set(title),
    fingerprint: Set(fingerprint),
    public_key: Set(public_key),
    can_push: Set(i32::from(can_push)),
    created_by_user_id: Set(created_by_user_id),
    last_used_at: Set(None),
    created_at: Set(now),
    updated_at: Set(now),
  }
  .insert(&tx)
  .await?;
  db.audit_snapshot(
    &tx,
    "project_deploy_keys",
    &deploy_key_json(&model),
    "insert",
    Some(created_by_user_id),
  )
  .await?;
  tx.commit().await?;
  Ok(model)
}

pub(super) async fn delete_deploy_key(
  db: &Db,
  key: project_deploy_keys::Model,
) -> Result<(), AppError> {
  let tx = db.transaction().await?;
  let current = project_deploy_keys::Entity::find_by_id(key.id)
    .filter(project_deploy_keys::Column::ProjectId.eq(key.project_id))
    .one(&tx)
    .await?;
  if let Some(current) = current {
    project_deploy_keys::Entity::delete_by_id(key.id)
      .exec(&tx)
      .await?;
    db.audit_snapshot(
      &tx,
      "project_deploy_keys",
      &deploy_key_json(&current),
      "delete",
      None,
    )
    .await?;
  }
  tx.commit().await?;
  Ok(())
}

pub(super) fn token_json(token: &project_access_tokens::Model) -> Value {
  json!({
    "id": token.id,
    "project_id": token.project_id,
    "kind": token.kind,
    "name": token.name,
    "username": token.username,
    "token": token.token,
    "scopes": token.scopes,
    "created_by_user_id": token.created_by_user_id,
    "expires_at": token.expires_at,
    "revoked_at": token.revoked_at,
    "last_used_at": token.last_used_at,
    "created_at": token.created_at,
    "updated_at": token.updated_at,
  })
}

pub(super) fn deploy_key_json(key: &project_deploy_keys::Model) -> Value {
  json!({
    "id": key.id,
    "project_id": key.project_id,
    "title": key.title,
    "fingerprint": key.fingerprint,
    "public_key": key.public_key,
    "can_push": key.can_push,
    "created_by_user_id": key.created_by_user_id,
    "last_used_at": key.last_used_at,
    "created_at": key.created_at,
    "updated_at": key.updated_at,
  })
}
