use super::tests::*;
use gity_entities::{projects, users};
use sea_orm::{ActiveModelTrait, Set};

#[tokio::test]
async fn ssh_key_candidates_require_active_creator_and_active_undeleted_project() {
  let (_dir, s) = fixture().await;
  let public_key =
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAINoV9GOEzZvKh6Egv81+heAA1fftTSTNP2joi2/vIyOl test";
  let (status, created) = json_req(
    &s,
    "POST",
    "/projects/20/deploy-keys",
    "alice",
    json!({"title":"SSH eligibility","public_key":public_key,"can_push":false}),
  )
  .await;
  assert_eq!(status, StatusCode::OK);
  let fingerprint = created["data"]["fingerprint"].as_str().unwrap();
  let candidates = || s.repository_access.authorized_keys(fingerprint);
  assert_eq!(candidates().await.unwrap().len(), 1);
  users::ActiveModel {
    id: Set(1),
    status: Set("disabled".into()),
    ..Default::default()
  }
  .update(&s.db.connection)
  .await
  .unwrap();
  assert!(candidates().await.unwrap().is_empty());
  users::ActiveModel {
    id: Set(1),
    status: Set("active".into()),
    ..Default::default()
  }
  .update(&s.db.connection)
  .await
  .unwrap();
  projects::ActiveModel {
    id: Set(20),
    status: Set("pending_delete".into()),
    ..Default::default()
  }
  .update(&s.db.connection)
  .await
  .unwrap();
  assert!(candidates().await.unwrap().is_empty());
  projects::ActiveModel {
    id: Set(20),
    status: Set("active".into()),
    deleted_at: Set(Some(chrono::Utc::now().naive_utc())),
    ..Default::default()
  }
  .update(&s.db.connection)
  .await
  .unwrap();
  assert!(candidates().await.unwrap().is_empty());
  projects::ActiveModel {
    id: Set(20),
    deleted_at: Set(None),
    ..Default::default()
  }
  .update(&s.db.connection)
  .await
  .unwrap();
  assert_eq!(candidates().await.unwrap().len(), 1);
  assert!(
    s.repository_access
      .authorized_keys("other")
      .await
      .unwrap()
      .is_empty()
  );
}
