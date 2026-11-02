use std::{borrow::Cow, str::FromStr};

use async_trait::async_trait;
use chrono::Utc;
use gity_entities::{oauth_authorization_codes, users};
use oxide_auth::{
  code_grant::accesstoken::{Authorization, Request as AccessRequest},
  primitives::{
    grant::{Extensions, Grant},
    issuer::{IssuedToken, RefreshedToken, TokenType},
    registrar::{BoundClient, ClientUrl, PreGrant, RegistrarError},
    scope::Scope,
  },
};
use oxide_auth_async::{
  code_grant::access_token::{Endpoint as AccessEndpoint, access_token},
  primitives::{Authorizer, Issuer, Registrar},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};

use super::tokens::{create_pair, digest, expiry};
use super::{OAuthError, OAuthService, TokenReply};

pub(super) async fn issue_from_code(
  service: &OAuthService,
  row: oauth_authorization_codes::Model,
  raw_code: &str,
) -> Result<TokenReply, OAuthError> {
  let scope = Scope::from_str(&row.scopes).map_err(|_| OAuthError::Invalid)?;
  let redirect_uri = url::Url::parse(&row.redirect_uri).map_err(|_| OAuthError::Invalid)?;
  let user = gity_entities::users::Entity::find_by_id(row.user_id)
    .one(&service.db)
    .await?
    .ok_or(OAuthError::Invalid)?;
  if user.status != "active"
    || (row.admin_mode && user.is_super_admin != 1)
    || row.expires_at <= Utc::now().naive_utc()
  {
    return Err(OAuthError::Invalid);
  }
  let grant = Grant {
    owner_id: row.user_id.to_string(),
    client_id: row.client_id.clone(),
    scope,
    redirect_uri,
    until: row.expires_at.and_utc(),
    extensions: Extensions::new(),
  };
  let mut endpoint = MachineEndpoint {
    service,
    code_hash: row.code_hash.clone(),
    grant,
    client_id: row.client_id.clone(),
    reply: None,
    extension: (),
  };
  let request = MachineRequest {
    code: raw_code.to_owned(),
    client_id: row.client_id.clone(),
    redirect_uri: row.redirect_uri.clone(),
  };
  let _bearer = access_token(&mut endpoint, &request)
    .await
    .map_err(|_| OAuthError::Invalid)?;
  endpoint.reply.ok_or(OAuthError::Invalid)
}

struct MachineRequest {
  code: String,
  client_id: String,
  redirect_uri: String,
}
impl AccessRequest for MachineRequest {
  fn valid(&self) -> bool {
    true
  }
  fn code(&self) -> Option<Cow<'_, str>> {
    Some(Cow::Borrowed(&self.code))
  }
  fn authorization(&self) -> Authorization<'_> {
    Authorization::None
  }
  fn client_id(&self) -> Option<Cow<'_, str>> {
    Some(Cow::Borrowed(&self.client_id))
  }
  fn redirect_uri(&self) -> Option<Cow<'_, str>> {
    Some(Cow::Borrowed(&self.redirect_uri))
  }
  fn grant_type(&self) -> Option<Cow<'_, str>> {
    Some(Cow::Borrowed("authorization_code"))
  }
  fn extension(&self, _: &str) -> Option<Cow<'_, str>> {
    None
  }
}

struct MachineEndpoint<'a> {
  service: &'a OAuthService,
  code_hash: String,
  grant: Grant,
  client_id: String,
  reply: Option<TokenReply>,
  extension: (),
}

#[async_trait]
impl Registrar for MachineEndpoint<'_> {
  async fn bound_redirect<'a>(&self, _: ClientUrl<'a>) -> Result<BoundClient<'a>, RegistrarError> {
    Err(RegistrarError::Unspecified)
  }
  async fn negotiate<'a>(
    &self,
    _: BoundClient<'a>,
    _: Option<Scope>,
  ) -> Result<PreGrant, RegistrarError> {
    Err(RegistrarError::Unspecified)
  }
  async fn check(&self, client_id: &str, passphrase: Option<&[u8]>) -> Result<(), RegistrarError> {
    if client_id == self.client_id && passphrase.is_none() {
      Ok(())
    } else {
      Err(RegistrarError::Unspecified)
    }
  }
}

#[async_trait]
impl Authorizer for MachineEndpoint<'_> {
  async fn authorize(&mut self, _: Grant) -> Result<String, ()> {
    Err(())
  }
  async fn extract(&mut self, token: &str) -> Result<Option<Grant>, ()> {
    Ok((digest(token) == self.code_hash).then(|| self.grant.clone()))
  }
}

#[async_trait]
impl Issuer for MachineEndpoint<'_> {
  async fn issue(&mut self, grant: Grant) -> Result<IssuedToken, ()> {
    let user_id = grant.owner_id.parse::<i64>().map_err(|_| ())?;
    let tx = self.service.db.begin().await.map_err(|_| ())?;
    let consumed = oauth_authorization_codes::Entity::update_many()
      .col_expr(
        oauth_authorization_codes::Column::RedeemedAt,
        sea_orm::sea_query::Expr::value(Utc::now().naive_utc()),
      )
      .filter(oauth_authorization_codes::Column::CodeHash.eq(&self.code_hash))
      .filter(oauth_authorization_codes::Column::RedeemedAt.is_null())
      .filter(oauth_authorization_codes::Column::ExpiresAt.gt(Utc::now().naive_utc()))
      .exec(&tx)
      .await
      .map_err(|_| ())?;
    if consumed.rows_affected != 1 {
      return Err(());
    }
    let row = oauth_authorization_codes::Entity::find()
      .filter(oauth_authorization_codes::Column::CodeHash.eq(&self.code_hash))
      .one(&tx)
      .await
      .map_err(|_| ())?
      .ok_or(())?;
    let user = users::Entity::find_by_id(user_id)
      .one(&tx)
      .await
      .map_err(|_| ())?
      .ok_or(())?;
    if user.status != "active" || (row.admin_mode && user.is_super_admin != 1) {
      return Err(());
    }
    let reply = create_pair(
      &tx,
      &row.client_id,
      row.user_id,
      &row.scopes,
      &row.resource,
      row.admin_mode,
      &self.service.config,
    )
    .await
    .map_err(|_| ())?;
    tx.commit().await.map_err(|_| ())?;
    self.reply = Some(reply.clone());
    Ok(IssuedToken {
      token: reply.access_token,
      refresh: Some(reply.refresh_token),
      until: expiry(self.service.config.access_token_ttl_seconds).and_utc(),
      token_type: TokenType::Bearer,
    })
  }
  async fn refresh(&mut self, _: &str, _: Grant) -> Result<RefreshedToken, ()> {
    Err(())
  }
  async fn recover_token(&mut self, _: &str) -> Result<Option<Grant>, ()> {
    Ok(None)
  }
  async fn recover_refresh(&mut self, _: &str) -> Result<Option<Grant>, ()> {
    Ok(None)
  }
}

impl AccessEndpoint for MachineEndpoint<'_> {
  fn registrar(&self) -> &(dyn Registrar + Sync) {
    self
  }
  fn authorizer(&mut self) -> &mut (dyn Authorizer + Send) {
    self
  }
  fn issuer(&mut self) -> &mut (dyn Issuer + Send) {
    self
  }
  fn extension(
    &mut self,
  ) -> &mut (dyn oxide_auth_async::code_grant::access_token::Extension + Send) {
    &mut self.extension
  }
}
