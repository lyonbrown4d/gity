use super::super::{AuthorizationRequest, ClientRegistration, OAuthService, TokenRequest};
use super::{OAuthError, parse_scopes};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::Utc;
use gity_core::OAuthConfig;
use gity_entities::users;
use sea_orm::{
  ActiveModelTrait, ActiveValue::Set, ColumnTrait, Database, EntityTrait, QueryFilter,
};
use sha2::{Digest, Sha256};
use url::Url;

#[test]
fn accepts_only_supported_scope_combinations() {
  assert_eq!(parse_scopes("mcp read_api").unwrap(), ["mcp", "read_api"]);
  assert!(parse_scopes("mcp write_api admin_mode").is_ok());
  assert!(matches!(parse_scopes("read_api"), Err(OAuthError::Invalid)));
  assert!(matches!(
    parse_scopes("mcp admin_mode:all"),
    Err(OAuthError::Invalid)
  ));
  assert!(matches!(parse_scopes("mcp mcp"), Err(OAuthError::Invalid)));
}

mod refresh_concurrency;

mod redirects;

#[tokio::test]
async fn issuer_rejects_unsafe_or_unroutable_base_urls() {
  let db = Database::connect("sqlite::memory:").await.unwrap();
  let config = OAuthConfig::default();
  assert!(
    OAuthService::new(
      db.clone(),
      config.clone(),
      "https://user:pass@example.test".into()
    )
    .is_err()
  );
  assert!(
    OAuthService::new(
      db.clone(),
      config.clone(),
      "https://example.test/prefix".into()
    )
    .is_err()
  );
  assert!(
    OAuthService::new(
      db.clone(),
      config.clone(),
      "https://example.test/?q=x".into()
    )
    .is_err()
  );
  assert!(OAuthService::new(db.clone(), config.clone(), "http://example.test".into()).is_err());
  assert!(OAuthService::new(db.clone(), config.clone(), "http://localhost:8080".into()).is_ok());
  assert!(OAuthService::new(db, config, "https://example.test".into()).is_ok());
}

#[tokio::test]
async fn persisted_authorization_code_refresh_and_reuse_revocation() {
  use gity_migrations::sea_orm_migration::MigratorTrait;

  let db = Database::connect("sqlite::memory:").await.unwrap();
  gity_migrations::Migrator::up(&db, None).await.unwrap();
  let now = Utc::now().naive_utc();
  users::ActiveModel {
    id: Set(gity_core::new_id()),
    username: Set("oauth-user".into()),
    display_name: Set("OAuth user".into()),
    email: Set(None),
    created_at: Set(now),
    updated_at: Set(now),
    is_super_admin: Set(1),
    status: Set("active".into()),
  }
  .insert(&db)
  .await
  .unwrap();
  let user = users::Entity::find().one(&db).await.unwrap().unwrap();
  let service = OAuthService::new(
    db.clone(),
    OAuthConfig::default(),
    "http://localhost:8080".into(),
  )
  .unwrap();
  let client: serde_json::Value = service
    .register_client(ClientRegistration {
      client_name: "test client".into(),
      redirect_uris: vec!["http://127.0.0.1:49152/callback".into()],
    })
    .await
    .unwrap();
  let client_id = client["client_id"].as_str().unwrap().to_owned();
  let verifier = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._~";
  let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
  let consent = service
    .begin_authorization(AuthorizationRequest {
      client_id: client_id.clone(),
      redirect_uri: "http://127.0.0.1:49152/callback".into(),
      response_type: "code".into(),
      code_challenge: challenge,
      code_challenge_method: "S256".into(),
      scope: "mcp read_api".into(),
      state: "state-value-with-enough-entropy".into(),
      resource: "http://localhost:8080/mcp".into(),
      nonce: None,
    })
    .await
    .unwrap();
  let request_id = Url::parse(&consent)
    .unwrap()
    .query_pairs()
    .find(|(k, _)| k == "request_id")
    .unwrap()
    .1
    .into_owned();
  service.pending_details(&request_id, user.id).await.unwrap();
  let callback = service
    .decide(&request_id, user.id, true, false)
    .await
    .unwrap();
  let code = Url::parse(&callback)
    .unwrap()
    .query_pairs()
    .find(|(k, _)| k == "code")
    .unwrap()
    .1
    .into_owned();
  let token = service
    .token(TokenRequest {
      grant_type: "authorization_code".into(),
      code: Some(code.clone()),
      refresh_token: None,
      client_id: client_id.clone(),
      redirect_uri: Some("http://127.0.0.1:49152/callback".into()),
      code_verifier: Some(verifier.into()),
      resource: Some("http://localhost:8080/mcp".into()),
    })
    .await
    .unwrap();
  let ordinary = service
    .validate_bearer(&token.access_token, "http://localhost:8080/mcp")
    .await
    .unwrap()
    .unwrap();
  assert!(!ordinary.is_super_admin);
  assert!(
    service
      .token(TokenRequest {
        grant_type: "authorization_code".into(),
        code: Some(code),
        refresh_token: None,
        client_id: client_id.clone(),
        redirect_uri: Some("http://127.0.0.1:49152/callback".into()),
        code_verifier: Some(verifier.into()),
        resource: Some("http://localhost:8080/mcp".into()),
      })
      .await
      .is_err()
  );

  let admin_consent = service
    .begin_authorization(AuthorizationRequest {
      client_id: client_id.clone(),
      redirect_uri: "http://127.0.0.1:49152/callback".into(),
      response_type: "code".into(),
      code_challenge: URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes())),
      code_challenge_method: "S256".into(),
      scope: "mcp admin_mode".into(),
      state: "second-state-value-with-enough-entropy".into(),
      resource: "http://localhost:8080/mcp".into(),
      nonce: None,
    })
    .await
    .unwrap();
  let admin_request_id = Url::parse(&admin_consent)
    .unwrap()
    .query_pairs()
    .find(|(k, _)| k == "request_id")
    .unwrap()
    .1
    .into_owned();
  service
    .pending_details(&admin_request_id, user.id)
    .await
    .unwrap();
  assert!(
    service
      .decide(&admin_request_id, user.id, true, false)
      .await
      .is_err()
  );
  let admin_callback = service
    .decide(&admin_request_id, user.id, true, true)
    .await
    .unwrap();
  let admin_code = Url::parse(&admin_callback)
    .unwrap()
    .query_pairs()
    .find(|(k, _)| k == "code")
    .unwrap()
    .1
    .into_owned();
  let admin_tokens = service
    .token(TokenRequest {
      grant_type: "authorization_code".into(),
      code: Some(admin_code),
      refresh_token: None,
      client_id: client_id.clone(),
      redirect_uri: Some("http://127.0.0.1:49152/callback".into()),
      code_verifier: Some(verifier.into()),
      resource: Some("http://localhost:8080/mcp".into()),
    })
    .await
    .unwrap();
  assert!(
    service
      .validate_bearer(&admin_tokens.access_token, "http://localhost:8080/mcp")
      .await
      .unwrap()
      .unwrap()
      .is_super_admin
  );
  users::Entity::update_many()
    .col_expr(
      users::Column::IsSuperAdmin,
      sea_orm::sea_query::Expr::value(0),
    )
    .filter(users::Column::Id.eq(user.id))
    .exec(&db)
    .await
    .unwrap();
  assert!(
    service
      .validate_bearer(&admin_tokens.access_token, "http://localhost:8080/mcp")
      .await
      .unwrap()
      .is_none()
  );

  let rotated = service
    .token(TokenRequest {
      grant_type: "refresh_token".into(),
      code: None,
      refresh_token: Some(token.refresh_token.clone()),
      client_id: client_id.clone(),
      redirect_uri: None,
      code_verifier: None,
      resource: Some("http://localhost:8080/mcp".into()),
    })
    .await
    .unwrap();
  assert!(
    service
      .validate_bearer(&rotated.access_token, "http://localhost:8080/mcp")
      .await
      .unwrap()
      .is_some()
  );
  assert!(
    service
      .token(TokenRequest {
        grant_type: "refresh_token".into(),
        code: None,
        refresh_token: Some(token.refresh_token),
        client_id,
        redirect_uri: None,
        code_verifier: None,
        resource: Some("http://localhost:8080/mcp".into()),
      })
      .await
      .is_err()
  );
  assert!(
    service
      .validate_bearer(&rotated.access_token, "http://localhost:8080/mcp")
      .await
      .unwrap()
      .is_none()
  );
}
