use super::tests::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
#[tokio::test]
async fn tokens_return_secret_once_enforce_project_scope_and_revoke() {
  let (_dir, s) = fixture().await;
  let (status, created) = json_req(
    &s,
    "POST",
    "/projects/20/deploy-tokens",
    "alice",
    json!({"name":"Publisher","scopes":["write_package"],"created_by_user_id":2}),
  )
  .await;
  assert_eq!(status, StatusCode::OK);
  let raw = created["data"]["token"].as_str().unwrap();
  assert_eq!(created["data"]["project_token"]["created_by_user_id"], "1");
  let token_id = created["data"]["project_token"]["id"]
    .as_str()
    .unwrap()
    .parse::<i64>()
    .unwrap();
  let row = gity_entities::project_access_tokens::Entity::find_by_id(token_id)
    .one(&s.db.connection)
    .await
    .unwrap()
    .unwrap();
  assert_ne!(row.token, raw);
  assert!(row.token.starts_with("sha256:"));
  assert!(authenticate_token(&s, &row.token).await.unwrap().is_none());
  let listed = json_req(&s, "GET", "/projects/20/deploy-tokens", "alice", json!({}))
    .await
    .1;
  assert!(listed["data"][0].get("token").is_none());
  assert_eq!(
    json_req(&s, "GET", "/projects/20/packages", raw, json!({}))
      .await
      .0,
    StatusCode::OK
  );
  let used = gity_entities::project_access_tokens::Entity::find_by_id(token_id)
    .one(&s.db.connection)
    .await
    .unwrap()
    .unwrap();
  assert!(used.last_used_at.is_some());
  assert_eq!(
    json_req(&s, "GET", "/projects/30/packages", raw, json!({}))
      .await
      .0,
    StatusCode::FORBIDDEN
  );
  let (_, registry_only) = json_req(
    &s,
    "POST",
    "/projects/20/deploy-tokens",
    "alice",
    json!({"name":"Container reader","scopes":["read_registry"]}),
  )
  .await;
  let registry_token = registry_only["data"]["token"].as_str().unwrap();
  assert_eq!(
    json_req(
      &s,
      "GET",
      "/projects/20/packages",
      registry_token,
      json!({})
    )
    .await
    .0,
    StatusCode::FORBIDDEN
  );
  assert_eq!(
    json_req(
      &s,
      "POST",
      "/projects/20/access-tokens",
      raw,
      json!({"name":"Escalate","scopes":["write_api"]})
    )
    .await
    .0,
    StatusCode::FORBIDDEN
  );
  let id = created["data"]["project_token"]["id"].as_str().unwrap();
  assert_eq!(
    json_req(
      &s,
      "DELETE",
      &format!("/projects/20/deploy-tokens/{id}"),
      "alice",
      json!({})
    )
    .await
    .0,
    StatusCode::OK
  );
  assert_eq!(
    json_req(&s, "GET", "/projects/20/packages", raw, json!({}))
      .await
      .0,
    StatusCode::UNAUTHORIZED
  );
}
#[tokio::test]
async fn token_expiry_and_masked_variable_updates_follow_persisted_utc_values() {
  let (_dir, s) = fixture().await;
  let (status, created) = json_req(
    &s,
    "POST",
    "/projects/20/access-tokens",
    "alice",
    json!({
        "name":"Expiring","scopes":["read_package"],"expires_at":"2000-01-01T00:00:00Z"
    }),
  )
  .await;
  assert_eq!(status, StatusCode::OK);
  let raw = created["data"]["token"].as_str().unwrap();
  assert_eq!(created["data"]["project_token"]["active"], false);
  assert_eq!(
    json_req(&s, "GET", "/projects/20/packages", raw, json!({}))
      .await
      .0,
    StatusCode::UNAUTHORIZED
  );
  let first = json_req(
    &s,
    "PATCH",
    "/projects/20/ci/variables",
    "alice",
    json!({"key":"build_token","value":"secret-value","masked":true,"protected":true}),
  )
  .await;
  assert_eq!(first.0, StatusCode::OK);
  assert_eq!(first.1["data"]["key"], "BUILD_TOKEN");
  assert_eq!(first.1["data"]["value"], "");
  let second = json_req(
    &s,
    "PATCH",
    "/projects/20/ci/variables",
    "alice",
    json!({"key":"BUILD_TOKEN","value":"changed-secret","masked":true}),
  )
  .await;
  assert_eq!(second.1["data"]["id"], first.1["data"]["id"]);
  let stored = s
    .db
    .one(
      "SELECT * FROM project_ci_variables WHERE project_id = ?",
      vec![20.into()],
    )
    .await
    .unwrap()
    .unwrap();
  assert_eq!(stored["value"], "changed-secret");
  assert_eq!(
    json_req(&s, "GET", "/projects/20/ci/variables", "bob", json!({}))
      .await
      .1["data"][0]["value"],
    ""
  );
  assert_eq!(
    json_req(
      &s,
      "PATCH",
      "/projects/20/ci/variables",
      "alice",
      json!({"key":"BUILD_TOKEN","value":"new\nsecret","masked":true})
    )
    .await
    .0,
    StatusCode::BAD_REQUEST
  );
  assert_eq!(
    json_req(
      &s,
      "GET",
      "/projects/20/ci/variables",
      "outsider",
      json!({})
    )
    .await
    .0,
    StatusCode::FORBIDDEN
  );
}

#[tokio::test]
async fn deploy_keys_keep_project_scope_and_audit_create_delete() {
  let (_dir, s) = fixture().await;
  let public_key =
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAINoV9GOEzZvKh6Egv81+heAA1fftTSTNP2joi2/vIyOl test";
  let (status, created) = json_req(
    &s,
    "POST",
    "/projects/20/deploy-keys",
    "alice",
    json!({"title":"CI read key","public_key":public_key,"can_push":false}),
  )
  .await;
  assert_eq!(status, StatusCode::OK);
  assert_eq!(created["data"]["can_push"], false);
  let id = created["data"]["id"].as_str().unwrap();
  assert_eq!(
    json_req(
      &s,
      "DELETE",
      &format!("/projects/30/deploy-keys/{id}"),
      "alice",
      json!({})
    )
    .await
    .0,
    StatusCode::NOT_FOUND
  );
  let listed = json_req(&s, "GET", "/projects/20/deploy-keys", "alice", json!({}))
    .await
    .1;
  assert_eq!(listed["data"][0]["id"], id);
  assert_eq!(listed["data"][0]["can_push"], false);
  assert_eq!(
    json_req(
      &s,
      "DELETE",
      &format!("/projects/20/deploy-keys/{id}"),
      "alice",
      json!({})
    )
    .await
    .0,
    StatusCode::OK
  );
  let audits = gity_entities::project_deploy_key_audits::Entity::find()
    .filter(gity_entities::project_deploy_key_audits::Column::ProjectId.eq(20))
    .order_by_asc(gity_entities::project_deploy_key_audits::Column::RevisionId)
    .all(&s.db.connection)
    .await
    .unwrap();
  assert_eq!(
    audits
      .iter()
      .map(|row| row.operation.as_str())
      .collect::<Vec<_>>(),
    ["insert", "delete"]
  );
}
