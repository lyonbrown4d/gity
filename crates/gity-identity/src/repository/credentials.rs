use chrono::{DateTime, NaiveDateTime, Utc};
use gity_entities::{auth_sessions, user_access_tokens, users};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait,
};

use super::{IdentityRepository, user_identity};
use crate::UserIdentity;

impl IdentityRepository {
  pub async fn personal_tokens(
    &self,
    user_id: i64,
  ) -> Result<Vec<user_access_tokens::Model>, DbErr> {
    user_access_tokens::Entity::find()
      .filter(user_access_tokens::Column::UserId.eq(user_id))
      .filter(user_access_tokens::Column::Name.is_not_in(["access", "refresh"]))
      .order_by_asc(user_access_tokens::Column::Id)
      .all(&self.connection)
      .await
  }

  pub async fn create_personal_token(
    &self,
    id: i64,
    user_id: i64,
    name: String,
    token_digest: String,
  ) -> Result<user_access_tokens::Model, DbErr> {
    let now = Utc::now().naive_utc();
    user_access_tokens::ActiveModel {
      id: Set(id),
      user_id: Set(user_id),
      name: Set(name),
      token: Set(token_digest),
      created_at: Set(now),
      updated_at: Set(now),
    }
    .insert(&self.connection)
    .await
  }

  /// Revokes one personal token owned by `user_id`; unused names such as
  /// `access`/`refresh` recorded by older schema versions stay untouched.
  pub async fn delete_personal_token(&self, user_id: i64, token_id: i64) -> Result<bool, DbErr> {
    let deleted = user_access_tokens::Entity::delete_many()
      .filter(user_access_tokens::Column::Id.eq(token_id))
      .filter(user_access_tokens::Column::UserId.eq(user_id))
      .filter(user_access_tokens::Column::Name.is_not_in(["access", "refresh"]))
      .exec(&self.connection)
      .await?
      .rows_affected;
    Ok(deleted > 0)
  }

  pub async fn create_session_pair(
    &self,
    user_id: i64,
    access_hash: String,
    refresh_hash: String,
    family: String,
    access_expires_at: DateTime<Utc>,
    refresh_expires_at: DateTime<Utc>,
  ) -> Result<(), DbErr> {
    let transaction = self.connection.begin().await?;
    for (token_hash, kind, expires_at) in [
      (access_hash, "access", access_expires_at.naive_utc()),
      (refresh_hash, "refresh", refresh_expires_at.naive_utc()),
    ] {
      auth_sessions::ActiveModel {
        token_hash: Set(token_hash),
        user_id: Set(user_id),
        kind: Set(kind.to_owned()),
        expires_at: Set(expires_at),
        family: Set(family.clone()),
      }
      .insert(&transaction)
      .await?;
    }
    transaction.commit().await
  }

  pub async fn consume_refresh_token(
    &self,
    digest: &str,
    now: NaiveDateTime,
  ) -> Result<Option<(UserIdentity, String)>, DbErr> {
    let transaction = self.connection.begin().await?;
    let session = auth_sessions::Entity::find_by_id(digest.to_owned())
      .filter(auth_sessions::Column::Kind.eq("refresh"))
      .filter(auth_sessions::Column::ExpiresAt.gt(now))
      .one(&transaction)
      .await?;
    let Some(session) = session else {
      transaction.rollback().await?;
      return Ok(None);
    };
    let deleted = auth_sessions::Entity::delete_many()
      .filter(auth_sessions::Column::TokenHash.eq(digest))
      .filter(auth_sessions::Column::Kind.eq("refresh"))
      .exec(&transaction)
      .await?
      .rows_affected;
    if deleted != 1 {
      transaction.rollback().await?;
      return Ok(None);
    }
    let user = users::Entity::find_by_id(session.user_id)
      .one(&transaction)
      .await?;
    transaction.commit().await?;
    Ok(user.map(|user| (user_identity(user), session.family)))
  }

  pub async fn session_family(&self, digest: &str) -> Result<Option<String>, DbErr> {
    auth_sessions::Entity::find_by_id(digest.to_owned())
      .one(&self.connection)
      .await
      .map(|session| session.map(|session| session.family))
  }

  pub async fn revoke_session_family(&self, family: &str) -> Result<(), DbErr> {
    auth_sessions::Entity::delete_many()
      .filter(auth_sessions::Column::Family.eq(family))
      .exec(&self.connection)
      .await
      .map(|_| ())
  }
}
