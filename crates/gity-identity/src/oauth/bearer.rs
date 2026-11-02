use super::tokens::digest;
use super::{OAuthError, OAuthPrincipal, OAuthService};
use chrono::Utc;
use gity_entities::{oauth_token_families, oauth_tokens, users};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

impl OAuthService {
  pub async fn validate_bearer(
    &self,
    token: &str,
    audience: &str,
  ) -> Result<Option<OAuthPrincipal>, OAuthError> {
    if !token.starts_with("gity_oauth_") || audience != self.resource {
      return Ok(None);
    }
    let row = oauth_tokens::Entity::find()
      .filter(oauth_tokens::Column::TokenHash.eq(digest(token)))
      .one(&self.db)
      .await?;
    let Some(row) = row else {
      return Ok(None);
    };
    if row.kind != "access"
      || row.revoked_at.is_some()
      || row.expires_at <= Utc::now().naive_utc()
      || row.resource != audience
    {
      return Ok(None);
    }
    if oauth_token_families::Entity::find()
      .filter(oauth_token_families::Column::FamilyKey.eq(&row.family_key))
      .filter(oauth_token_families::Column::RevokedAt.is_null())
      .one(&self.db)
      .await?
      .is_none()
    {
      return Ok(None);
    }
    let Some(user) = users::Entity::find_by_id(row.user_id).one(&self.db).await? else {
      return Ok(None);
    };
    if user.status != "active" || (row.admin_mode && user.is_super_admin != 1) {
      return Ok(None);
    }
    Ok(Some(OAuthPrincipal {
      user_id: user.id,
      username: user.username,
      is_super_admin: row.admin_mode && user.is_super_admin == 1,
      scopes: row.scopes.split_whitespace().map(str::to_owned).collect(),
    }))
  }
}
