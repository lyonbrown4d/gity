use super::{OpenSearchEngine, kind_name};
use crate::global::{CandidatePage, EngineQuery, ResourceKind, SearchDocument, matches};
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};

const RAW_PAGE_SIZE: usize = 32;
const MAX_RAW_HITS: usize = 50_000;

pub(super) async fn run(engine: &OpenSearchEngine, request: EngineQuery) -> Result<CandidatePage> {
  let query = request.query.trim().to_owned();
  let grams = query_grams(&query);
  if grams.is_empty() {
    return Ok(CandidatePage::default());
  }
  let must = grams
    .iter()
    .map(|gram| json!({"term":{"grams":gram}}))
    .collect::<Vec<_>>();
  let mut filters = vec![
    json!({"terms":{"project_id":request.project_ids}}),
    json!({"term":{"kind":kind_name(request.kind)}}),
  ];
  let scope_keys = request
    .namespaces
    .iter()
    .map(|(project, namespace)| format!("{project}:{namespace}"))
    .collect::<Vec<_>>();
  filters.push(json!({"terms":{"scope_key":scope_keys}}));
  if request.kind == ResourceKind::Issue {
    let mut visibility = vec![json!({"term":{"confidential":false}})];
    if !request.confidential_projects.is_empty() {
      visibility.push(json!({"terms":{"project_id":request.confidential_projects}}));
    }
    if !request.confidential_issue_ids.is_empty() {
      visibility.push(json!({"terms":{"resource_id":request.confidential_issue_ids.iter().map(i64::to_string).collect::<Vec<_>>()}}));
    }
    filters.push(json!({"bool":{"should":visibility,"minimum_should_match":1}}));
  }
  let query_body = json!({"bool":{"filter":filters,"must":must,"should":[{"match":{"title":{"query":query,"boost":3.0}}}]}});
  let mut pit = open_pit(engine).await?;
  let result = pages(engine, &request, &query, query_body, &mut pit).await;
  let cleanup = close_pit(engine, pit).await;
  match (result, cleanup) {
    (Err(error), _) => Err(error),
    (Ok(_), Err(error)) => Err(error),
    (Ok(page), Ok(())) => Ok(page),
  }
}

async fn open_pit(engine: &OpenSearchEngine) -> Result<String> {
  let response = engine
    .request(
      reqwest::Method::POST,
      &format!("/{}/_search/point_in_time?keep_alive=1m", engine.index),
      None,
    )
    .await?;
  if !response.status().is_success() {
    bail!(
      "OpenSearch snapshot creation failed with status {}",
      response.status()
    );
  }
  let value: Value = response
    .json()
    .await
    .context("OpenSearch snapshot response was invalid")?;
  value
    .get("pit_id")
    .and_then(Value::as_str)
    .map(str::to_owned)
    .context("OpenSearch snapshot response lacked an ID")
}

async fn close_pit(engine: &OpenSearchEngine, pit: String) -> Result<()> {
  let response = engine
    .request(
      reqwest::Method::DELETE,
      "/_search/point_in_time",
      Some(json!({"pit_id":[pit]})),
    )
    .await?;
  if !response.status().is_success() {
    bail!(
      "OpenSearch snapshot cleanup failed with status {}",
      response.status()
    );
  }
  Ok(())
}

async fn pages(
  engine: &OpenSearchEngine,
  request: &EngineQuery,
  query: &str,
  query_body: Value,
  pit: &mut String,
) -> Result<CandidatePage> {
  let required = request
    .offset
    .saturating_add(request.limit)
    .saturating_add(1);
  let mut found: Vec<SearchDocument> = Vec::new();
  let mut scanned = 0;
  let mut search_after: Option<Value> = None;
  let mut exhausted = false;
  let mut cursor_lost = false;
  while scanned < MAX_RAW_HITS && found.len() < required {
    let size = RAW_PAGE_SIZE.min(MAX_RAW_HITS - scanned);
    let mut body = json!({"size":size,"track_total_hits":false,"query":query_body.clone(),"sort":[{"_score":"desc"},{"search_sort":"asc"}],"pit":{"id":pit,"keep_alive":"1m"},"_source":{"excludes":["grams"]}});
    if let Some(cursor) = search_after.take() {
      body["search_after"] = cursor;
    }
    let response = engine
      .request(reqwest::Method::POST, "/_search", Some(body))
      .await?;
    if !response.status().is_success() {
      bail!("OpenSearch query failed with status {}", response.status());
    }
    let value: Value = response
      .json()
      .await
      .context("OpenSearch query response was invalid")?;
    validate_response(&value)?;
    if let Some(next_pit) = value.get("pit_id").and_then(Value::as_str) {
      *pit = next_pit.to_owned();
    }
    let hits = value
      .pointer("/hits/hits")
      .and_then(Value::as_array)
      .context("OpenSearch query response lacked hits")?;
    if hits.is_empty() {
      exhausted = true;
      break;
    }
    scanned += hits.len();
    let Some(last) = hits.last() else {
      exhausted = true;
      break;
    };
    search_after = last.get("sort").cloned();
    if search_after.is_none() {
      cursor_lost = true;
    }
    for hit in hits {
      let source = hit
        .get("_source")
        .context("OpenSearch hit lacked a source document")?;
      let document = serde_json::from_value::<SearchDocument>(source.clone())
        .context("OpenSearch hit had an invalid document")?;
      if matches(query, &document.title)
        || matches(query, &document.body)
        || matches(query, &document.path)
      {
        // Candidate pages retain identity and navigation metadata; the platform hydrates display text.
        let mut document = document;
        document.title = String::new();
        document.body = String::new();
        found.push(document);
        if found.len() >= required {
          break;
        }
      }
    }
    if hits.len() < size {
      exhausted = true;
      break;
    }
    if cursor_lost {
      break;
    }
  }
  let end = request.offset.saturating_add(request.limit);
  let has_more = found.len() > end;
  let partial = found.len() < required && !exhausted && (scanned >= MAX_RAW_HITS || cursor_lost);
  let documents = found
    .into_iter()
    .skip(request.offset)
    .take(request.limit)
    .collect();
  Ok(CandidatePage {
    documents,
    has_more,
    partial,
  })
}

fn validate_response(value: &Value) -> Result<()> {
  let timed_out = value
    .get("timed_out")
    .and_then(Value::as_bool)
    .unwrap_or(false);
  let failed = value
    .pointer("/_shards/failed")
    .and_then(Value::as_u64)
    .unwrap_or(0);
  if timed_out || failed > 0 {
    bail!("OpenSearch query was incomplete (timed_out={timed_out}, failed_shards={failed})");
  }
  Ok(())
}

fn query_grams(query: &str) -> Vec<String> {
  let chars = query.to_lowercase().chars().collect::<Vec<_>>();
  if chars.is_empty() {
    return Vec::new();
  }
  if chars.len() < 3 {
    return vec![chars.iter().collect()];
  }
  chars
    .windows(3)
    .map(|gram| gram.iter().collect())
    .collect::<std::collections::BTreeSet<_>>()
    .into_iter()
    .collect()
}
