use gity_search::{
  BackendKind, GlobalIndex, GlobalSearchConfig,
  global::{EngineQuery, ResourceKind, SearchDocument},
};

fn document(
  key: &str,
  project: i64,
  kind: ResourceKind,
  title: &str,
  confidential: bool,
) -> SearchDocument {
  SearchDocument {
    key: key.into(),
    project_id: project,
    kind,
    resource_id: key.into(),
    version: "v1".into(),
    title: title.into(),
    body: "body text".into(),
    path: "src/file.rs".into(),
    iid: None,
    line_start: 0,
    confidential,
  }
}

fn request(query: &str, kind: ResourceKind) -> EngineQuery {
  EngineQuery {
    query: query.into(),
    project_ids: vec![1, 2],
    namespaces: vec![(1, "generation-1".into()), (2, "generation-1".into())],
    confidential_projects: vec![],
    confidential_issue_ids: vec![],
    kind,
    offset: 0,
    limit: 20,
  }
}

#[tokio::test]
async fn embedded_filters_unicode_and_replaces_project_atomically() {
  let root = tempfile::tempdir().unwrap();
  let index = GlobalIndex::new(root.path(), GlobalSearchConfig::default()).unwrap();
  index
    .replace_project(
      1,
      "generation-1",
      vec![
        document("code-1", 1, ResourceKind::Code, "Café repository", false),
        document("issue-public", 1, ResourceKind::Issue, "Café issue", false),
        document("issue-private", 1, ResourceKind::Issue, "Café secret", true),
      ],
    )
    .await
    .unwrap();
  index
    .replace_project(
      2,
      "generation-1",
      vec![document(
        "wrong-project",
        2,
        ResourceKind::Code,
        "Café docs",
        false,
      )],
    )
    .await
    .unwrap();

  let mut code_request = request("CAFÉ", ResourceKind::Code);
  code_request.project_ids = vec![1];
  code_request.namespaces = vec![(1, "generation-1".into())];
  let code = index.query(code_request).await.unwrap();
  assert_eq!(
    code
      .documents
      .iter()
      .map(|doc| doc.key.as_str())
      .collect::<Vec<_>>(),
    vec!["code-1"]
  );
  assert_eq!(code.documents[0].title.capacity(), 0);
  assert_eq!(code.documents[0].body.capacity(), 0);
  let issues = index
    .query(request("café", ResourceKind::Issue))
    .await
    .unwrap();
  assert_eq!(
    issues
      .documents
      .iter()
      .map(|doc| doc.key.as_str())
      .collect::<Vec<_>>(),
    vec!["issue-public"]
  );

  let mut scoped = request("café", ResourceKind::Issue);
  scoped.confidential_projects = vec![1];
  assert_eq!(index.query(scoped).await.unwrap().documents.len(), 2);
  index
    .replace_project(
      1,
      "generation-2",
      vec![document(
        "new-code",
        1,
        ResourceKind::Code,
        "Café generation",
        false,
      )],
    )
    .await
    .unwrap();
  let mut new_generation = request("café", ResourceKind::Code);
  new_generation.namespaces = vec![(1, "generation-2".into())];
  assert_eq!(
    index.query(new_generation).await.unwrap().documents[0].key,
    "new-code"
  );
  index.delete_namespace(1, "generation-1").await.unwrap();
  let remaining = index
    .query(request("café", ResourceKind::Code))
    .await
    .unwrap();
  assert_eq!(remaining.documents.len(), 1);
  assert_eq!(remaining.documents[0].key, "wrong-project");
}

#[tokio::test]
async fn configuration_rejects_credentials_embedded_in_endpoint() {
  let root = tempfile::tempdir().unwrap();
  let config = GlobalSearchConfig {
    backend: BackendKind::OpenSearch,
    endpoint: "https://user:secret@example.test".into(),
    ..GlobalSearchConfig::default()
  };
  let error = GlobalIndex::new(root.path(), config).err().unwrap();
  assert!(!error.to_string().contains("secret"));
}

#[tokio::test]
async fn candidate_snapshot_limit_is_bounded_at_one_thousand() {
  let root = tempfile::tempdir().unwrap();
  let index = GlobalIndex::new(root.path(), GlobalSearchConfig::default()).unwrap();
  let mut bounded = request("needle", ResourceKind::Code);
  bounded.limit = 1_000;
  assert!(!index.query(bounded).await.unwrap().partial);
  let mut oversized = request("needle", ResourceKind::Code);
  oversized.limit = 1_001;
  assert!(index.query(oversized).await.is_err());
}
