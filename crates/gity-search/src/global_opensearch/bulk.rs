use super::OpenSearchEngine;
use crate::global::SearchDocument;
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const MAX_BULK_DOCUMENTS: usize = 200;
const MAX_BULK_BYTES: usize = 5 * 1024 * 1024;

pub(super) async fn write(
  engine: &OpenSearchEngine,
  project_id: i64,
  namespace: &str,
  documents: Vec<SearchDocument>,
) -> Result<()> {
  let mut lines = Vec::new();
  let mut bytes = 0;
  let mut count = 0;
  for document in documents {
    let mut pair = encode(project_id, namespace, &document)?;
    let pair_bytes = pair.iter().map(String::len).sum::<usize>();
    if pair_bytes > MAX_BULK_BYTES {
      bail!("a global search document exceeds the OpenSearch bulk batch limit");
    }
    if count > 0 && (count == MAX_BULK_DOCUMENTS || bytes + pair_bytes > MAX_BULK_BYTES) {
      send(engine, std::mem::take(&mut lines)).await?;
      count = 0;
      bytes = 0;
    }
    bytes += pair_bytes;
    count += 1;
    lines.append(&mut pair);
  }
  if !lines.is_empty() {
    send(engine, lines).await?;
  }
  Ok(())
}

fn encode(project_id: i64, namespace: &str, document: &SearchDocument) -> Result<Vec<String>> {
  let key_digest = hex_digest(&document.key);
  let id = serde_json::to_string(&format!("{project_id}:{namespace}:{key_digest}"))?;
  let action = format!("{{\"index\":{{\"_id\":{id}}}}}\n");
  let source = serde_json::to_string(&json!({
    "key": document.key,
    "project_id": document.project_id,
    "namespace": namespace,
    "scope_key": format!("{project_id}:{namespace}"),
    "search_sort": format!("{project_id}:{namespace}:{}", document.key),
    "kind": kind_name(document.kind),
    "resource_id": document.resource_id,
    "version": document.version,
    "title": document.title,
    "body": document.body,
    "path": document.path,
    "iid": document.iid,
    "line_start": document.line_start,
    "confidential": document.confidential,
    "grams": format!("{} {} {}", document.title, document.path, document.body).to_lowercase(),
  }))?;
  Ok(vec![action, format!("{source}\n")])
}

fn hex_digest(value: &str) -> String {
  Sha256::digest(value.as_bytes())
    .iter()
    .map(|byte| format!("{byte:02x}"))
    .collect()
}

async fn send(engine: &OpenSearchEngine, lines: Vec<String>) -> Result<()> {
  let path = format!("/{}/_bulk?refresh=wait_for", engine.index);
  let body = lines.concat();
  let mut request = engine
    .client
    .post(format!("{}{}", engine.base, path))
    .header(reqwest::header::CONTENT_TYPE, "application/x-ndjson")
    .body(body);
  if let Some((username, password)) = &engine.auth {
    request = request.basic_auth(username, Some(password));
  }
  let response = request
    .send()
    .await
    .context("OpenSearch bulk request failed")?;
  if !response.status().is_success() {
    bail!(
      "OpenSearch bulk request failed with status {}",
      response.status()
    );
  }
  let result: Value = response
    .json()
    .await
    .context("OpenSearch bulk response was invalid")?;
  if result
    .get("errors")
    .and_then(Value::as_bool)
    .unwrap_or(true)
  {
    let failed = result
      .get("items")
      .and_then(Value::as_array)
      .map(|items| {
        items
          .iter()
          .filter(|item| item.get("index").and_then(|op| op.get("error")).is_some())
          .count()
      })
      .unwrap_or(0);
    bail!("OpenSearch bulk indexing reported errors ({failed} failed items)");
  }
  Ok(())
}

fn kind_name(kind: crate::global::ResourceKind) -> &'static str {
  match kind {
    crate::global::ResourceKind::Code => "code",
    crate::global::ResourceKind::Issue => "issue",
  }
}
