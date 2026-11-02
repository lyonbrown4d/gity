use chrono::{NaiveDateTime, Utc};
use gity_entities::{
  auth_sessions, bootstrap_claim, organization_members, organizations, user_access_tokens,
  user_passwords, user_ssh_keys, users,
};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter, Set, TransactionTrait,
};

use super::{IdentityRepository, user_lock::lock_user};
use crate::UserProfileChanges;

impl IdentityRepository {
  pub async fn create_user(
    &self,
    id: i64,
    username: String,
    display_name: String,
    email: Option<String>,
    is_super_admin: bool,
  ) -> Result<users::Model, DbErr> {
    let now = Utc::now().naive_utc();
    users::ActiveModel {
      id: Set(id),
      username: Set(username),
      display_name: Set(display_name),
      email: Set(email),
      created_at: Set(now),
      updated_at: Set(now),
      is_super_admin: Set(if is_super_admin { 1 } else { 0 }),
      status: Set("active".to_owned()),
    }
    .insert(&self.connection)
    .await
  }

  pub async fn register_account(
    &self,
    id: i64,
    organization_id: i64,
    membership_id: i64,
    username: String,
    password_hash: String,
    timestamp: NaiveDateTime,
  ) -> Result<(users::Model, bool), DbErr> {
    let transaction = self.connection.begin().await?;
    let is_first = users::Entity::find().one(&transaction).await?.is_none();
    if is_first {
      bootstrap_claim::ActiveModel { id: Set(1) }
        .insert(&transaction)
        .await?;
    }
    let user = users::ActiveModel {
      id: Set(id),
      username: Set(username.clone()),
      display_name: Set(username.clone()),
      email: Set(Some(format!("{username}@local.gity"))),
      created_at: Set(timestamp),
      updated_at: Set(timestamp),
      is_super_admin: Set(if is_first { 1 } else { 0 }),
      status: Set("active".to_owned()),
    }
    .insert(&transaction)
    .await?;
    user_passwords::ActiveModel {
      user_id: Set(id),
      password_hash: Set(password_hash),
    }
    .insert(&transaction)
    .await?;
    let path = format!("{}-{id}", username.to_lowercase());
    organizations::ActiveModel {
      id: Set(organization_id),
      name: Set(username.clone()),
      path_key: Set(path.clone()),
      full_path: Set(path),
      description: Set(Some("Personal namespace".to_owned())),
      visibility: Set("private".to_owned()),
      created_at: Set(timestamp),
      updated_at: Set(timestamp),
    }
    .insert(&transaction)
    .await?;
    organization_members::ActiveModel {
      id: Set(membership_id),
      organization_id: Set(organization_id),
      user_id: Set(id),
      role: Set("owner".to_owned()),
      created_at: Set(timestamp),
      updated_at: Set(timestamp),
    }
    .insert(&transaction)
    .await?;
    transaction.commit().await?;
    Ok((user, is_first))
  }

  pub async fn set_password_and_revoke_sessions(
    &self,
    user_id: i64,
    password_hash: String,
  ) -> Result<bool, DbErr> {
    let transaction = self.connection.begin().await?;
    if users::Entity::find_by_id(user_id)
      .one(&transaction)
      .await?
      .is_none()
    {
      transaction.rollback().await?;
      return Ok(false);
    }
    user_passwords::Entity::delete_by_id(user_id)
      .exec(&transaction)
      .await?;
    user_passwords::ActiveModel {
      user_id: Set(user_id),
      password_hash: Set(password_hash),
    }
    .insert(&transaction)
    .await?;
    auth_sessions::Entity::delete_many()
      .filter(auth_sessions::Column::UserId.eq(user_id))
      .exec(&transaction)
      .await?;
    transaction.commit().await?;
    Ok(true)
  }

  pub async fn update_user_profile(
    &self,
    user_id: i64,
    changes: UserProfileChanges,
  ) -> Result<Option<users::Model>, DbErr> {
    let transaction = self.connection.begin().await?;
    if users::Entity::find_by_id(user_id)
      .one(&transaction)
      .await?
      .is_none()
    {
      transaction.rollback().await?;
      return Ok(None);
    }
    let revoke_sessions =
      changes.password_hash.is_some() || changes.status.as_deref() == Some("disabled");
    let model = users::ActiveModel {
      id: Set(user_id),
      username: changes.username.map(Set).unwrap_or_default(),
      display_name: changes.display_name.map(Set).unwrap_or_default(),
      email: changes.email.map(Set).unwrap_or_default(),
      is_super_admin: changes
        .is_super_admin
        .map(|value| Set(if value { 1 } else { 0 }))
        .unwrap_or_default(),
      status: changes.status.map(Set).unwrap_or_default(),
      updated_at: Set(Utc::now().naive_utc()),
      ..Default::default()
    };
    let updated = model.update(&transaction).await?;
    if let Some(password_hash) = changes.password_hash {
      user_passwords::Entity::delete_by_id(user_id)
        .exec(&transaction)
        .await?;
      user_passwords::ActiveModel {
        user_id: Set(user_id),
        password_hash: Set(password_hash),
      }
      .insert(&transaction)
      .await?;
    }
    if revoke_sessions {
      auth_sessions::Entity::delete_many()
        .filter(auth_sessions::Column::UserId.eq(user_id))
        .exec(&transaction)
        .await?;
    }
    transaction.commit().await?;
    Ok(Some(updated))
  }

  pub async fn delete_user_and_credentials(&self, user_id: i64) -> Result<bool, DbErr> {
    let transaction = self.connection.begin().await?;
    if lock_user(&transaction, user_id).await?.is_none() {
      transaction.rollback().await?;
      return Ok(false);
    }
    for result in [
      user_ssh_keys::Entity::delete_many()
        .filter(user_ssh_keys::Column::UserId.eq(user_id))
        .exec(&transaction)
        .await,
      auth_sessions::Entity::delete_many()
        .filter(auth_sessions::Column::UserId.eq(user_id))
        .exec(&transaction)
        .await,
      user_passwords::Entity::delete_by_id(user_id)
        .exec(&transaction)
        .await,
      user_access_tokens::Entity::delete_many()
        .filter(user_access_tokens::Column::UserId.eq(user_id))
        .exec(&transaction)
        .await,
    ] {
      result?;
    }
    let deleted = users::Entity::delete_by_id(user_id)
      .exec(&transaction)
      .await?
      .rows_affected
      > 0;
    if deleted {
      transaction.commit().await?;
    } else {
      transaction.rollback().await?;
    }
    Ok(deleted)
  }
}
