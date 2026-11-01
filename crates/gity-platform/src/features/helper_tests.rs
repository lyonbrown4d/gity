use super::*;
#[tokio::test]
async fn project_lookup_and_collection_replacement_are_isolated_and_atomic() {
  let dir = tempfile::tempdir().unwrap();
  let mut settings = gity_core::Settings::load();
  settings.repo_root = dir.path().join("repos");
  settings.storage_root = dir.path().join("storage");
  tokio::fs::create_dir_all(&settings.repo_root)
    .await
    .unwrap();
  let db = gity_core::Db::connect("sqlite::memory:").await.unwrap();
  // Minimal real tables focus the test on collection rollback and tenant isolation.
  use sea_orm::ConnectionTrait;
  let schema = concat!(
    "CREATE TABLE project_issues(id INTEGER PRIMARY KEY, project_id INTEGER, iid INTEGER); ",
    "CREATE TABLE project_issue_labels(id INTEGER PRIMARY KEY,project_issue_id INTEGER,",
    "name TEXT UNIQUE NOT NULL,color TEXT,created_at TEXT,updated_at TEXT);"
  );
  db.connection.execute_unprepared(schema).await.unwrap();
  db.execute(
    "INSERT INTO project_issues(id,project_id,iid) VALUES (10,1,1),(20,2,1)",
    vec![],
  )
  .await
  .unwrap();
  let state = AppState {
    git_cluster: None,
    identity: gity_identity::IdentityModule::new(fluxdi::Shared::new(db.connection.clone()))
      .repository()
      .unwrap(),
    artifacts: gity_artifacts::ArtifactModule::new(
      fluxdi::Shared::new(db.connection.clone()),
      fluxdi::Shared::new(gity_core::Storage::local(&settings.storage_root).unwrap()),
    )
    .service()
    .unwrap(),
    search: gity_search::SearchIndex::new(
      dir.path().join("search"),
      gity_git::GitService::new(&settings.repo_root, &settings.git_bin),
      gity_search::Limits::default(),
    )
    .unwrap(),
    git: gity_git::GitService::new(&settings.repo_root, &settings.git_bin),
    storage: gity_core::Storage::local(&settings.storage_root).unwrap(),
    settings: std::sync::Arc::new(settings),
    db,
  };
  assert_eq!(
    number(
      &scoped(&state, "project_issues", 2, 1, true).await.unwrap(),
      "id"
    ),
    20
  );
  assert!(
    scoped(&state, "project_issues", 2, 10, false)
      .await
      .is_err()
  );
  replace(
    &state,
    "project_issue_labels",
    "project_issue_id",
    10,
    None,
    vec![fields(
      json!({"project_issue_id":10,"name":"original","color":"red"}),
    )],
  )
  .await
  .unwrap();
  let invalid = vec![
    fields(json!({"project_issue_id":10,"name":"duplicate"})),
    fields(json!({"project_issue_id":10,"name":"duplicate"})),
  ];
  assert!(
    replace(
      &state,
      "project_issue_labels",
      "project_issue_id",
      10,
      None,
      invalid
    )
    .await
    .is_err()
  );
  let rows = state
    .db
    .all(
      "SELECT name FROM project_issue_labels WHERE project_issue_id = ?",
      vec![10.into()],
    )
    .await
    .unwrap();
  assert_eq!(rows, vec![json!({"name":"original"})]);
}
#[test]
fn issue_wire_contract_uses_project_local_iid() {
  let view = externalize(issues::view(
    json!({"id":99,"iid":3,"project_id":4,"author_user_id":5,"title":"Issue","state":"opened"}),
  ));
  assert_eq!(view["id"], "3");
  assert_eq!(view["repository_id"], "4");
  assert_eq!(view["number"], 3);
  assert_eq!(view["status"], "open");
  assert_eq!(view["author_user_id"], "5");
}
