use crate::{
  BackendKind, GlobalSearchConfig,
  global::{CandidatePage, EngineQuery, SearchDocument},
  global_opensearch, global_tantivy,
};
use anyhow::{Result, bail};
use std::{path::Path, sync::Arc};

#[derive(Clone)]
pub struct GlobalIndex {
  inner: Arc<Inner>,
}

enum Inner {
  Tantivy(global_tantivy::TantivyEngine),
  OpenSearch(global_opensearch::OpenSearchEngine),
}

impl GlobalIndex {
  pub fn new(root: &Path, config: GlobalSearchConfig) -> Result<Self> {
    config.validate()?;
    let inner = match config.backend {
      BackendKind::Tantivy => Inner::Tantivy(global_tantivy::TantivyEngine::new(root)?),
      BackendKind::OpenSearch => {
        Inner::OpenSearch(global_opensearch::OpenSearchEngine::new(config)?)
      }
    };
    Ok(Self {
      inner: Arc::new(inner),
    })
  }

  pub async fn replace_project(
    &self,
    project_id: i64,
    namespace: &str,
    documents: Vec<SearchDocument>,
  ) -> Result<()> {
    validate_namespace(namespace)?;
    if project_id <= 0 || documents.iter().any(|doc| doc.project_id != project_id) {
      bail!("search documents must belong to the target project");
    }
    match self.inner.as_ref() {
      Inner::Tantivy(engine) => {
        engine
          .replace_project(project_id, namespace, documents)
          .await
      }
      Inner::OpenSearch(engine) => {
        engine
          .replace_project(project_id, namespace, documents)
          .await
      }
    }
  }

  pub async fn delete_namespace(&self, project_id: i64, namespace: &str) -> Result<()> {
    if project_id <= 0 {
      bail!("positive project ID required");
    }
    validate_namespace(namespace)?;
    match self.inner.as_ref() {
      Inner::Tantivy(engine) => engine.delete_namespace(project_id, namespace).await,
      Inner::OpenSearch(engine) => engine.delete_namespace(project_id, namespace).await,
    }
  }

  pub async fn query(&self, query: EngineQuery) -> Result<CandidatePage> {
    if query.limit == 0 || query.offset > 10_000 || query.limit > 1_000 {
      bail!("invalid global search page");
    }
    if query.project_ids.is_empty()
      || query.project_ids.iter().any(|id| *id <= 0)
      || query.namespaces.is_empty()
      || query
        .namespaces
        .iter()
        .any(|(id, name)| *id <= 0 || validate_namespace(name).is_err())
    {
      return Ok(CandidatePage::default());
    }
    match self.inner.as_ref() {
      Inner::Tantivy(engine) => engine.query(query).await,
      Inner::OpenSearch(engine) => engine.query(query).await,
    }
  }
}

fn validate_namespace(namespace: &str) -> Result<()> {
  if namespace.is_empty()
    || namespace.len() > 128
    || !namespace
      .bytes()
      .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
  {
    bail!("invalid search index namespace");
  }
  Ok(())
}
