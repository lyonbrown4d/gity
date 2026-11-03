use crate::{
  GlobalSearchConfig,
  global::{CandidatePage, EngineQuery, ResourceKind, SearchDocument},
};
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};

mod bulk;
mod query;
use std::sync::Arc;
use tokio::sync::OnceCell;

#[derive(Clone)]
pub(super) struct OpenSearchEngine {
  client: reqwest::Client,
  base: String,
  index: String,
  auth: Option<(String, String)>,
  init: Arc<OnceCell<()>>,
}

impl OpenSearchEngine {
  pub fn new(config: GlobalSearchConfig) -> Result<Self> {
    let client = reqwest::Client::builder()
      .timeout(config.timeout())
      .redirect(reqwest::redirect::Policy::none())
      .build()?;
    let base = config.endpoint.trim_end_matches('/').to_owned();
    let auth = (!config.username.is_empty() || !config.password.is_empty())
      .then_some((config.username, config.password));
    Ok(Self {
      client,
      base,
      index: config.index,
      auth,
      init: Arc::new(OnceCell::new()),
    })
  }

  async fn request(
    &self,
    method: reqwest::Method,
    path: &str,
    body: Option<Value>,
  ) -> Result<reqwest::Response> {
    let url = format!("{}{path}", self.base);
    let mut request = self.client.request(method, url);
    if let Some((username, password)) = &self.auth {
      request = request.basic_auth(username, Some(password));
    }
    if let Some(body) = body {
      request = request.json(&body);
    }
    request.send().await.context("OpenSearch request failed")
  }

  async fn ensure_index(&self) -> Result<()> {
    self.init.get_or_try_init(|| async {
      let path = format!("/{}", self.index);
      let response = self.request(reqwest::Method::GET, &path, None).await?;
      if response.status().is_success() {
        let existing: Value = response.json().await.context("OpenSearch index metadata was invalid")?;
        validate_mapping(&existing, &self.index)?;
        return Ok(());
      }
      if response.status() != reqwest::StatusCode::NOT_FOUND { return Err(anyhow::anyhow!("OpenSearch index check failed with status {}", response.status())); }
      let mapping = json!({"settings":{"index":{"max_ngram_diff":2},"analysis":{"tokenizer":{"gity_grams":{"type":"ngram","min_gram":1,"max_gram":3}},"analyzer":{"gity_substring":{"type":"custom","tokenizer":"gity_grams","filter":["lowercase"]}}}},"mappings":{"_meta":{"gity_global_schema":1},"dynamic":"strict","properties":{"key":{"type":"keyword"},"project_id":{"type":"long"},"namespace":{"type":"keyword"},"scope_key":{"type":"keyword"},"search_sort":{"type":"keyword"},"kind":{"type":"keyword"},"resource_id":{"type":"keyword"},"version":{"type":"keyword"},"title":{"type":"text","fields":{"exact":{"type":"keyword","ignore_above":512}}},"body":{"type":"text"},"path":{"type":"text"},"iid":{"type":"long"},"line_start":{"type":"long"},"confidential":{"type":"boolean"},"grams":{"type":"text","analyzer":"gity_substring","search_analyzer":"keyword"}}}});
      let response = self.request(reqwest::Method::PUT, &path, Some(mapping)).await?;
      if !response.status().is_success() { return Err(anyhow::anyhow!("OpenSearch index initialization failed with status {}", response.status())); }
      Ok(())
    }).await.map(|_| ())
  }

  pub async fn replace_project(
    &self,
    project_id: i64,
    namespace: &str,
    documents: Vec<SearchDocument>,
  ) -> Result<()> {
    self.ensure_index().await?;
    let delete_path = format!(
      "/{}/_delete_by_query?refresh=true&conflicts=proceed",
      self.index
    );
    let response = self
      .request(
        reqwest::Method::POST,
        &delete_path,
        Some(json!({"query":{"bool":{"filter":[{"term":{"project_id":project_id}},{"term":{"namespace":namespace}}]}}})),
      )
      .await?;
    if !response.status().is_success() {
      bail!(
        "OpenSearch project replacement delete failed with status {}",
        response.status()
      );
    }
    check_delete_result(
      response
        .json()
        .await
        .context("OpenSearch delete response was invalid")?,
    )?;
    bulk::write(self, project_id, namespace, documents).await
  }

  pub async fn delete_namespace(&self, project_id: i64, namespace: &str) -> Result<()> {
    self.ensure_index().await?;
    let path = format!("/{}/_delete_by_query?refresh=true", self.index);
    let response = self.request(reqwest::Method::POST, &path, Some(json!({"query":{"bool":{"filter":[{"term":{"project_id":project_id}},{"term":{"namespace":namespace}}]}}}))).await?;
    if !response.status().is_success() {
      bail!(
        "OpenSearch namespace deletion failed with status {}",
        response.status()
      );
    }
    check_delete_result(
      response
        .json()
        .await
        .context("OpenSearch delete response was invalid")?,
    )?;
    Ok(())
  }

  pub async fn query(&self, request: EngineQuery) -> Result<CandidatePage> {
    self.ensure_index().await?;
    query::run(self, request).await
  }
}

fn kind_name(kind: ResourceKind) -> &'static str {
  match kind {
    ResourceKind::Code => "code",
    ResourceKind::Issue => "issue",
  }
}

fn check_delete_result(body: Value) -> Result<()> {
  let failures = body
    .get("failures")
    .and_then(Value::as_array)
    .map_or(0, Vec::len);
  let timed_out = body
    .get("timed_out")
    .and_then(Value::as_bool)
    .unwrap_or(false);
  if failures > 0 || timed_out {
    bail!(
      "OpenSearch namespace deletion was incomplete ({failures} failures, timed_out={timed_out})"
    );
  }
  Ok(())
}

fn validate_mapping(body: &Value, index: &str) -> Result<()> {
  let mapping = body.get(index).and_then(|value| value.get("mappings"));
  let schema = mapping
    .and_then(|value| value.pointer("/_meta/gity_global_schema"))
    .and_then(Value::as_i64);
  let properties = mapping.and_then(|value| value.get("properties"));
  let has_required_fields = [
    "project_id",
    "namespace",
    "scope_key",
    "search_sort",
    "kind",
    "resource_id",
    "confidential",
    "grams",
  ]
  .iter()
  .all(|field| properties.and_then(|value| value.get(field)).is_some());
  if schema != Some(1) || !has_required_fields {
    bail!("OpenSearch index mapping is incompatible; configure a fresh global search index");
  }
  Ok(())
}
