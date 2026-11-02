use crate::{OAuthService, TokenRequest};
use chrono::Utc;
use gity_core::OAuthConfig;
use gity_entities::{oauth_token_families, oauth_tokens, users};
use sea_orm::{
  ActiveModelTrait, ActiveValue::Set, ColumnTrait, Database, EntityTrait, QueryFilter,
};

#[tokio::test]
async fn concurrent_refresh_replay_revokes_winning_descendants() {
  use gity_migrations::sea_orm_migration::MigratorTrait;

  let directory = tempfile::tempdir().unwrap();
  let path = directory.path().join("oauth-concurrency.sqlite");
  let dsn = format!("sqlite://{}?mode=rwc", path.display());
  let first_db = Database::connect(&dsn).await.unwrap();
  gity_migrations::Migrator::up(&first_db, None)
    .await
    .unwrap();
  let second_db = Database::connect(&dsn).await.unwrap();
  let now = Utc::now().naive_utc();
  users::ActiveModel {
    id: Set(gity_core::new_id()),
    username: Set("refresh-race".into()),
    display_name: Set("Refresh race".into()),
    email: Set(None),
    created_at: Set(now),
    updated_at: Set(now),
    is_super_admin: Set(0),
    status: Set("active".into()),
  }
  .insert(&first_db)
  .await
  .unwrap();
  let user = users::Entity::find()
    .filter(users::Column::Username.eq("refresh-race"))
    .one(&first_db)
    .await
    .unwrap()
    .unwrap();
  let family_key = uuid::Uuid::now_v7().to_string();
  oauth_token_families::ActiveModel {
    id: Set(gity_core::new_id()),
    family_key: Set(family_key.clone()),
    created_at: Set(now),
    revoked_at: Set(None),
  }
  .insert(&first_db)
  .await
  .unwrap();
  let raw_refresh = "gity_oauth_single-use-race-token";
  oauth_tokens::ActiveModel {
    id: Set(gity_core::new_id()),
    token_hash: Set(super::super::digest(raw_refresh)),
    family_key: Set(family_key.clone()),
    client_id: Set("race-client".into()),
    user_id: Set(user.id),
    kind: Set("refresh".into()),
    scopes: Set("mcp read_api".into()),
    resource: Set("http://localhost:8080/mcp".into()),
    admin_mode: Set(false),
    expires_at: Set(now + chrono::Duration::days(1)),
    revoked_at: Set(None),
    replaced_by_hash: Set(None),
  }
  .insert(&first_db)
  .await
  .unwrap();
  let first = OAuthService::new(
    first_db.clone(),
    OAuthConfig::default(),
    "http://localhost:8080".into(),
  )
  .unwrap();
  let second = OAuthService::new(
    second_db,
    OAuthConfig::default(),
    "http://localhost:8080".into(),
  )
  .unwrap();
  let request = || TokenRequest {
    grant_type: "refresh_token".into(),
    code: None,
    refresh_token: Some(raw_refresh.into()),
    client_id: "race-client".into(),
    redirect_uri: None,
    code_verifier: None,
    resource: Some("http://localhost:8080/mcp".into()),
  };
  let (left, right) = tokio::join!(first.token(request()), second.token(request()));
  let winner = left
    .ok()
    .or_else(|| right.ok())
    .expect("one concurrent refresh must win");
  assert!(
    first
      .validate_bearer(&winner.access_token, "http://localhost:8080/mcp")
      .await
      .unwrap()
      .is_none()
  );
  let late = "gity_oauth_late-descendant-after-revocation";
  oauth_tokens::ActiveModel {
    id: Set(gity_core::new_id()),
    token_hash: Set(super::super::digest(late)),
    family_key: Set(family_key),
    client_id: Set("race-client".into()),
    user_id: Set(user.id),
    kind: Set("access".into()),
    scopes: Set("mcp read_api".into()),
    resource: Set("http://localhost:8080/mcp".into()),
    admin_mode: Set(false),
    expires_at: Set(now + chrono::Duration::hours(1)),
    revoked_at: Set(None),
    replaced_by_hash: Set(None),
  }
  .insert(&first_db)
  .await
  .unwrap();
  assert!(
    first
      .validate_bearer(late, "http://localhost:8080/mcp")
      .await
      .unwrap()
      .is_none(),
    "the persistent grant marker must reject descendants missed by a token-row snapshot"
  );
}
