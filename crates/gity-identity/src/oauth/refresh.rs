use super::tokens::{digest, expiry, secret};
use super::{OAuthError, OAuthService, TokenReply, TokenRequest};
use chrono::Utc;
use gity_entities::{oauth_token_families, oauth_tokens, users};
use sea_orm::{
  ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, TransactionTrait,
};

impl OAuthService {
  pub(super) async fn rotate_refresh(
    &self,
    request: TokenRequest,
  ) -> Result<TokenReply, OAuthError> {
    let raw = request.refresh_token.ok_or(OAuthError::Invalid)?;
    let hash = digest(&raw);
    let new_refresh = secret();
    let new_hash = digest(&new_refresh);
    let tx = self.db.begin().await?;
    // Write before reading: SQLite cannot upgrade a stale read snapshot while
    // another connection commits a rotation. This CAS also serializes redemption
    // on PostgreSQL/MySQL without a separate, process-local lock.
    let mut claim = oauth_tokens::Entity::update_many()
      .col_expr(
        oauth_tokens::Column::RevokedAt,
        sea_orm::sea_query::Expr::value(Utc::now().naive_utc()),
      )
      .col_expr(
        oauth_tokens::Column::ReplacedByHash,
        sea_orm::sea_query::Expr::value(new_hash.clone()),
      )
      .filter(oauth_tokens::Column::TokenHash.eq(&hash))
      .filter(oauth_tokens::Column::Kind.eq("refresh"))
      .filter(oauth_tokens::Column::ClientId.eq(&request.client_id))
      .filter(oauth_tokens::Column::RevokedAt.is_null())
      .filter(oauth_tokens::Column::ExpiresAt.gt(Utc::now().naive_utc()));
    if let Some(resource) = &request.resource {
      claim = claim.filter(oauth_tokens::Column::Resource.eq(resource));
    }
    match claim.exec(&tx).await {
      Ok(result) if result.rows_affected == 1 => {}
      Ok(_) => {
        tx.rollback().await?;
        self.reject_reuse(&hash, &request.client_id, false).await?;
        return Err(OAuthError::Expired);
      }
      Err(error) => {
        tx.rollback().await?;
        // An uncertain redemption must fail closed for this known credential.
        self.reject_reuse(&hash, &request.client_id, true).await?;
        return Err(error.into());
      }
    }
    let row = oauth_tokens::Entity::find()
      .filter(oauth_tokens::Column::TokenHash.eq(&hash))
      .one(&tx)
      .await?
      .ok_or(OAuthError::Invalid)?;
    let family = oauth_token_families::Entity::find()
      .filter(oauth_token_families::Column::FamilyKey.eq(&row.family_key))
      .filter(oauth_token_families::Column::RevokedAt.is_null())
      .one(&tx)
      .await?;
    if family.is_none() {
      return Err(OAuthError::Expired);
    }
    let user = users::Entity::find_by_id(row.user_id)
      .one(&tx)
      .await?
      .ok_or(OAuthError::Invalid)?;
    if user.status != "active" || (row.admin_mode && user.is_super_admin != 1) {
      return Err(OAuthError::Invalid);
    }
    let access_token = format!("gity_oauth_{}", secret());
    for (hash, kind, ttl) in [
      (
        digest(&access_token),
        "access",
        self.config.access_token_ttl_seconds,
      ),
      (new_hash, "refresh", self.config.refresh_token_ttl_seconds),
    ] {
      oauth_tokens::ActiveModel {
        id: Set(gity_core::new_id()),
        token_hash: Set(hash),
        family_key: Set(row.family_key.clone()),
        client_id: Set(row.client_id.clone()),
        user_id: Set(row.user_id),
        kind: Set(kind.into()),
        scopes: Set(row.scopes.clone()),
        resource: Set(row.resource.clone()),
        admin_mode: Set(row.admin_mode),
        expires_at: Set(expiry(ttl)),
        revoked_at: Set(None),
        replaced_by_hash: Set(None),
      }
      .insert(&tx)
      .await?;
    }
    tx.commit().await?;
    Ok(TokenReply {
      access_token,
      token_type: "Bearer",
      expires_in: self.config.access_token_ttl_seconds,
      refresh_token: new_refresh,
      scope: row.scopes,
    })
  }

  async fn reject_reuse(
    &self,
    hash: &str,
    client_id: &str,
    uncertain: bool,
  ) -> Result<(), OAuthError> {
    let row = oauth_tokens::Entity::find()
      .filter(oauth_tokens::Column::TokenHash.eq(hash))
      .filter(oauth_tokens::Column::Kind.eq("refresh"))
      .filter(oauth_tokens::Column::ClientId.eq(client_id))
      .one(&self.db)
      .await?;
    if let Some(row) = row
      && (uncertain || row.revoked_at.is_some())
    {
      self.revoke_family_reuse(&row.family_key).await?;
    }
    Ok(())
  }

  /// RFC 7009: revoking either credential invalidates its entire authorization grant.
  pub async fn revoke(&self, token: &str) -> Result<(), OAuthError> {
    let row = oauth_tokens::Entity::find()
      .filter(oauth_tokens::Column::TokenHash.eq(digest(token)))
      .one(&self.db)
      .await?;
    if let Some(row) = row {
      self.revoke_family_reuse(&row.family_key).await?;
    }
    Ok(())
  }

  async fn revoke_family_reuse(&self, family_key: &str) -> Result<(), OAuthError> {
    let tx = self.db.begin().await?;
    let now = Utc::now().naive_utc();
    // The grant marker remains authoritative even if a concurrent transaction
    // inserts another descendant after the token-row update's snapshot.
    oauth_token_families::Entity::update_many()
      .col_expr(
        oauth_token_families::Column::RevokedAt,
        sea_orm::sea_query::Expr::value(now),
      )
      .filter(oauth_token_families::Column::FamilyKey.eq(family_key))
      .filter(oauth_token_families::Column::RevokedAt.is_null())
      .exec(&tx)
      .await?;
    oauth_tokens::Entity::update_many()
      .col_expr(
        oauth_tokens::Column::RevokedAt,
        sea_orm::sea_query::Expr::value(now),
      )
      .filter(oauth_tokens::Column::FamilyKey.eq(family_key))
      .filter(oauth_tokens::Column::RevokedAt.is_null())
      .exec(&tx)
      .await?;
    tx.commit().await?;
    Ok(())
  }
}
