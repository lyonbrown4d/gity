use gity_search::{
  BackendKind, GlobalIndex, GlobalSearchConfig,
  global::{EngineQuery, ResourceKind, SearchDocument},
};
use std::time::{SystemTime, UNIX_EPOCH};

fn document(key: String, title: String, body: String) -> SearchDocument {
  SearchDocument {
    key: key.clone(),
    project_id: 37,
    kind: ResourceKind::Code,
    resource_id: key,
    version: "v1".into(),
    title,
    body,
    path: "src/test.rs".into(),
    iid: None,
    line_start: 1,
    confidential: false,
  }
}

fn issue(project_id: i64, id: i64, confidential: bool) -> SearchDocument {
  SearchDocument {
    key: format!("issue-{id}"),
    project_id,
    kind: ResourceKind::Issue,
    resource_id: id.to_string(),
    version: "2026-11-03T00:00:00Z".into(),
    title: "needle issue".into(),
    body: "issue body".into(),
    path: String::new(),
    iid: Some(id),
    line_start: 0,
    confidential,
  }
}

#[tokio::test]
#[ignore = "requires an isolated OpenSearch endpoint in GITY_TEST_OPENSEARCH_ENDPOINT"]
async fn opensearch_initializes_batches_and_scans_past_false_candidates() {
  let endpoint = std::env::var("GITY_TEST_OPENSEARCH_ENDPOINT").expect("test endpoint is required");
  let username = std::env::var("GITY_TEST_OPENSEARCH_USERNAME").unwrap_or_default();
  let password = std::env::var("GITY_TEST_OPENSEARCH_PASSWORD").unwrap_or_default();
  let suffix = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let index_name = format!("gity-it-{}-{suffix:x}", std::process::id());
  let config = GlobalSearchConfig {
    backend: BackendKind::OpenSearch,
    endpoint: endpoint.clone(),
    index: index_name.clone(),
    username: username.clone(),
    password: password.clone(),
    timeout_seconds: 30,
    allow_insecure_http: true,
  };
  let root = tempfile::tempdir().unwrap();
  let index = GlobalIndex::new(root.path(), config).unwrap();
  let namespace = "integration-generation";
  let mut documents = (0..600)
    .map(|number| {
      document(
        format!("a-{number:04}"),
        "nee eed edl dle ".repeat(8),
        "noise".into(),
      )
    })
    .collect::<Vec<_>>();
  documents.push(document(
    "z-exact".into(),
    "exact title".into(),
    "needle in body".into(),
  ));
  let long_key = format!("long-{}", "目录".repeat(400));
  let mut long_path_document = document(long_key.clone(), "pathprobe".into(), "content".into());
  long_path_document.path = format!("src/{}.rs", "文件".repeat(400));
  documents.push(long_path_document);
  documents.push(issue(37, 7100, false));
  documents.push(issue(37, 7101, true));
  documents.push(issue(37, 7102, true));
  index
    .replace_project(37, namespace, documents)
    .await
    .unwrap();
  index
    .replace_project(38, "other-generation", vec![issue(38, 7103, false)])
    .await
    .unwrap();
  let page = index
    .query(EngineQuery {
      query: "needle".into(),
      project_ids: vec![37],
      namespaces: vec![(37, namespace.into())],
      confidential_projects: vec![],
      confidential_issue_ids: vec![],
      kind: ResourceKind::Code,
      offset: 0,
      limit: 1,
    })
    .await
    .unwrap();
  assert_eq!(
    page
      .documents
      .iter()
      .map(|doc| doc.key.as_str())
      .collect::<Vec<_>>(),
    vec!["z-exact"]
  );
  assert_eq!(page.documents[0].title.capacity(), 0);
  assert_eq!(page.documents[0].body.capacity(), 0);
  assert!(!page.has_more);
  assert!(!page.partial);
  let long_page = index
    .query(EngineQuery {
      query: "pathprobe".into(),
      project_ids: vec![37],
      namespaces: vec![(37, namespace.into())],
      confidential_projects: vec![],
      confidential_issue_ids: vec![],
      kind: ResourceKind::Code,
      offset: 0,
      limit: 1,
    })
    .await
    .unwrap();
  assert_eq!(long_page.documents[0].key, long_key);
  assert!(long_page.documents[0].path.chars().count() >= 700);
  let issue_query =
    |confidential_projects: Vec<i64>, confidential_issue_ids: Vec<i64>| EngineQuery {
      query: "needle".into(),
      project_ids: vec![37, 38],
      namespaces: vec![(37, namespace.into())],
      confidential_projects,
      confidential_issue_ids,
      kind: ResourceKind::Issue,
      offset: 0,
      limit: 10,
    };
  let public = index.query(issue_query(vec![], vec![])).await.unwrap();
  assert_eq!(
    public
      .documents
      .iter()
      .map(|doc| doc.resource_id.as_str())
      .collect::<Vec<_>>(),
    vec!["7100"]
  );
  let issue_override = index.query(issue_query(vec![], vec![7102])).await.unwrap();
  assert_eq!(
    issue_override
      .documents
      .iter()
      .map(|doc| doc.resource_id.as_str())
      .collect::<Vec<_>>(),
    vec!["7100", "7102"]
  );
  let project_override = index.query(issue_query(vec![37], vec![])).await.unwrap();
  assert_eq!(
    project_override
      .documents
      .iter()
      .map(|doc| doc.project_id)
      .collect::<Vec<_>>(),
    vec![37, 37, 37]
  );
  index.delete_namespace(37, namespace).await.unwrap();
  index
    .delete_namespace(38, "other-generation")
    .await
    .unwrap();

  let client = reqwest::Client::new();
  let mut cleanup = client.delete(format!("{}/{index_name}", endpoint.trim_end_matches('/')));
  if !username.is_empty() || !password.is_empty() {
    cleanup = cleanup.basic_auth(username, Some(password));
  }
  let response = cleanup.send().await.unwrap();
  assert!(
    response.status().is_success(),
    "test-owned index cleanup failed"
  );
}
