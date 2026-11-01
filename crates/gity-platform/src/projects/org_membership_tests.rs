use crate::{auth_tests::request, features::test_support::fixture};
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn organization_member_identity_permissions_duplicates_and_last_owner_are_enforced() {
  let (_dir, state) = fixture().await;
  for (organization, owner, name) in [(10, 1, "Team"), (11, 2, "Other")] {
    if organization == 11 {
      state.db.insert("organizations", json!({"id":organization,"name":name,"path_key":"other","full_path":"other","visibility":"private"}).as_object().unwrap().clone()).await.unwrap();
    }
    state
      .db
      .insert(
        "organization_members",
        json!({"organization_id":organization,"user_id":owner,"role":"owner"})
          .as_object()
          .unwrap()
          .clone(),
      )
      .await
      .unwrap();
  }
  let (status, added) = request(
    &state,
    "POST",
    "/orgs/10/members",
    "alice",
    json!({"user_id":"2","role":"developer"}),
  )
  .await;
  assert_eq!(status, StatusCode::OK, "{added}");
  assert_eq!(added["data"]["username"], "bob");
  assert_eq!(added["data"]["email"], "bob@test.local");
  assert_eq!(added["data"]["role"], "developer");
  let listed = request(&state, "GET", "/orgs/10/members", "alice", json!({}))
    .await
    .1;
  assert_eq!(listed["data"].as_array().unwrap().len(), 2);
  assert!(
    listed["data"]
      .as_array()
      .unwrap()
      .iter()
      .any(|row| row["email"] == "bob@test.local")
  );
  assert_eq!(
    request(
      &state,
      "POST",
      "/orgs/10/members",
      "alice",
      json!({"user_id":2,"role":"owner"})
    )
    .await
    .0,
    StatusCode::CONFLICT
  );
  assert_eq!(
    request(
      &state,
      "POST",
      "/orgs/10/members",
      "outsider",
      json!({"user_id":3,"role":"developer"})
    )
    .await
    .0,
    StatusCode::FORBIDDEN
  );
  assert_eq!(
    request(
      &state,
      "PATCH",
      "/orgs/11/members/2",
      "alice",
      json!({"role":"developer"})
    )
    .await
    .0,
    StatusCode::FORBIDDEN
  );
  assert_eq!(
    request(
      &state,
      "PATCH",
      "/orgs/10/members/1",
      "alice",
      json!({"role":"developer"})
    )
    .await
    .0,
    StatusCode::CONFLICT
  );
  assert_eq!(
    request(&state, "DELETE", "/orgs/10/members/1", "alice", json!({}))
      .await
      .0,
    StatusCode::CONFLICT
  );
  assert_eq!(
    request(
      &state,
      "PATCH",
      "/orgs/10/members/3",
      "alice",
      json!({"role":"developer"})
    )
    .await
    .0,
    StatusCode::NOT_FOUND
  );
  assert_eq!(
    request(
      &state,
      "POST",
      "/orgs/10/members",
      "alice",
      json!({"user_id":99,"role":"developer"})
    )
    .await
    .0,
    StatusCode::NOT_FOUND
  );
  assert_eq!(
    request(
      &state,
      "PATCH",
      "/orgs/10/members/2",
      "alice",
      json!({"role":"owner"})
    )
    .await
    .0,
    StatusCode::OK
  );
  state
    .db
    .update(
      "users",
      1,
      json!({"is_super_admin":1}).as_object().unwrap().clone(),
    )
    .await
    .unwrap();
  let (remove, demote) = tokio::join!(
    request(&state, "DELETE", "/orgs/10/members/1", "alice", json!({})),
    request(
      &state,
      "PATCH",
      "/orgs/10/members/2",
      "alice",
      json!({"role":"developer"})
    ),
  );
  let mut statuses = vec![remove.0.as_u16(), demote.0.as_u16()];
  statuses.sort();
  assert_eq!(statuses, vec![200, 409]);
  assert_eq!(
    state
      .db
      .all(
        "SELECT user_id FROM organization_members WHERE organization_id=10 AND role='owner'",
        vec![]
      )
      .await
      .unwrap()
      .len(),
    1
  );
  assert_eq!(
    state
      .db
      .one(
        "SELECT role FROM organization_members WHERE organization_id=11 AND user_id=2",
        vec![]
      )
      .await
      .unwrap()
      .unwrap()["role"],
    "owner"
  );
}

#[tokio::test]
async fn organization_name_updates_require_owner_and_validate_before_persisting() {
  let (_dir, state) = fixture().await;
  state
    .db
    .insert(
      "organization_members",
      json!({"organization_id":10,"user_id":1,"role":"owner"})
        .as_object()
        .unwrap()
        .clone(),
    )
    .await
    .unwrap();
  let (status, updated) = request(
    &state,
    "PATCH",
    "/orgs/10",
    "alice",
    json!({"key":"team","name":"Updated Team"}),
  )
  .await;
  assert_eq!(status, StatusCode::OK);
  assert_eq!(updated["data"]["name"], "Updated Team");
  assert_eq!(updated["data"]["key"], "team");
  assert_eq!(
    request(
      &state,
      "PATCH",
      "/orgs/10",
      "bob",
      json!({"name":"Forbidden"})
    )
    .await
    .0,
    StatusCode::FORBIDDEN
  );
  assert_eq!(
    request(&state, "PATCH", "/orgs/10", "alice", json!({"name":"   "}))
      .await
      .0,
    StatusCode::BAD_REQUEST
  );
  assert_eq!(
    state
      .db
      .one("SELECT name FROM organizations WHERE id=10", vec![])
      .await
      .unwrap()
      .unwrap()["name"],
    "Updated Team"
  );
}
