use chrono::Utc;
use gity_entities::{oauth_authorization_codes, oauth_clients, oauth_pending_requests, users};
use sea_orm::{
  ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
  TransactionTrait,
};
use url::Url;

use super::protocol::{digest, expiry, random_secret};
use super::repository::PendingAuthorization;
use super::{OAuthError, OAuthService};

impl OAuthService {
  pub async fn pending_details(
    &self,
    key: &str,
    user_id: i64,
  ) -> Result<PendingAuthorization, OAuthError> {
    let row = self.pending(key).await?;
    if row.expires_at <= Utc::now().naive_utc() || row.status != "pending" {
      return Err(OAuthError::Expired);
    }
    if let Some(bound) = row.user_id
      && bound != user_id
    {
      return Err(OAuthError::Invalid);
    }
    let mut update = row.clone().into_active_model();
    if row.user_id.is_none() {
      let bound = oauth_pending_requests::Entity::update_many()
        .col_expr(
          oauth_pending_requests::Column::UserId,
          sea_orm::sea_query::Expr::value(user_id),
        )
        .filter(oauth_pending_requests::Column::Id.eq(row.id))
        .filter(oauth_pending_requests::Column::UserId.is_null())
        .filter(oauth_pending_requests::Column::Status.eq("pending"))
        .exec(&self.db)
        .await?;
      if bound.rows_affected == 0 {
        let current = self.pending(key).await?;
        if current.user_id != Some(user_id) {
          return Err(OAuthError::Invalid);
        }
      }
      update.user_id = Set(Some(user_id));
    }
    let client = oauth_clients::Entity::find()
      .filter(oauth_clients::Column::ClientId.eq(row.client_id))
      .one(&self.db)
      .await?
      .ok_or(OAuthError::Invalid)?;
    Ok(PendingAuthorization {
      request_id: row.request_key,
      client_name: client.name,
      scopes: row.scopes.split_whitespace().map(str::to_owned).collect(),
      resource: row.resource,
      requests_admin_mode: row.admin_mode,
    })
  }

  pub async fn decide(
    &self,
    key: &str,
    user_id: i64,
    approve: bool,
    admin_reauthenticated: bool,
  ) -> Result<String, OAuthError> {
    let tx = self.db.begin().await?;
    let row = oauth_pending_requests::Entity::find()
      .filter(oauth_pending_requests::Column::RequestKey.eq(key))
      .one(&tx)
      .await?
      .ok_or(OAuthError::Invalid)?;
    if row.user_id != Some(user_id)
      || row.status != "pending"
      || row.expires_at <= Utc::now().naive_utc()
    {
      return Err(OAuthError::Expired);
    }
    if !approve {
      let denied = oauth_pending_requests::Entity::update_many()
        .col_expr(
          oauth_pending_requests::Column::Status,
          sea_orm::sea_query::Expr::value("denied"),
        )
        .filter(oauth_pending_requests::Column::Id.eq(row.id))
        .filter(oauth_pending_requests::Column::Status.eq("pending"))
        .exec(&tx)
        .await?;
      if denied.rows_affected != 1 {
        return Err(OAuthError::Expired);
      }
      tx.commit().await?;
      let mut callback = Url::parse(&row.redirect_uri).map_err(|_| OAuthError::Invalid)?;
      callback
        .query_pairs_mut()
        .append_pair("error", "access_denied")
        .append_pair("state", &row.state);
      return Ok(callback.to_string());
    }
    let user = users::Entity::find_by_id(user_id)
      .one(&tx)
      .await?
      .ok_or(OAuthError::Invalid)?;
    if user.status != "active"
      || (row.admin_mode && (user.is_super_admin != 1 || !admin_reauthenticated))
    {
      return Err(OAuthError::Invalid);
    }
    let approved = oauth_pending_requests::Entity::update_many()
      .col_expr(
        oauth_pending_requests::Column::Status,
        sea_orm::sea_query::Expr::value("approved"),
      )
      .filter(oauth_pending_requests::Column::Id.eq(row.id))
      .filter(oauth_pending_requests::Column::Status.eq("pending"))
      .filter(oauth_pending_requests::Column::UserId.eq(user_id))
      .exec(&tx)
      .await?;
    if approved.rows_affected != 1 {
      return Err(OAuthError::Expired);
    }
    let code = random_secret(32);
    let code_hash = digest(&code);
    oauth_authorization_codes::ActiveModel {
      id: Set(gity_core::new_id()),
      code_hash: Set(code_hash),
      client_id: Set(row.client_id.clone()),
      user_id: Set(user_id),
      redirect_uri: Set(row.redirect_uri.clone()),
      code_challenge: Set(row.code_challenge.clone()),
      scopes: Set(row.scopes.clone()),
      resource: Set(row.resource.clone()),
      admin_mode: Set(row.admin_mode),
      expires_at: Set(expiry(self.config.authorization_code_ttl_seconds)),
      redeemed_at: Set(None),
    }
    .insert(&tx)
    .await?;
    tx.commit().await?;
    let mut callback = Url::parse(&row.redirect_uri).map_err(|_| OAuthError::Invalid)?;
    callback
      .query_pairs_mut()
      .append_pair("code", &code)
      .append_pair("state", &row.state);
    Ok(callback.to_string())
  }

  async fn pending(&self, key: &str) -> Result<oauth_pending_requests::Model, OAuthError> {
    oauth_pending_requests::Entity::find()
      .filter(oauth_pending_requests::Column::RequestKey.eq(key))
      .one(&self.db)
      .await?
      .ok_or(OAuthError::Invalid)
  }
}
