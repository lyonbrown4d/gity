use super::{request, text};
use futures::{StreamExt, stream};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeSet;

const PAGE_SIZE: u64 = 100;
const INVALID: &str = "The user directory could not be loaded. Try again.";
const CHANGED: &str = "The user directory changed while loading. Refresh and try again.";

#[derive(Deserialize)]
struct Page {
  items: Vec<Value>,
  total: u64,
  page: u64,
  page_size: u64,
}

#[derive(Debug, PartialEq)]
enum PageError {
  Request(String),
  Invalid,
  Changed,
  Cancelled,
}

fn parse_page(value: Value, expected_page: u64) -> Result<Page, PageError> {
  let page: Page = serde_json::from_value(value).map_err(|_| PageError::Invalid)?;
  if page.page != expected_page
    || page.page_size != PAGE_SIZE
    || page.items.len() as u64 > PAGE_SIZE
    || page.items.len() as u64 > page.total
  {
    return Err(PageError::Invalid);
  }
  Ok(page)
}

fn append_page(
  rows: &mut Vec<Value>,
  seen: &mut BTreeSet<String>,
  page: Page,
  total: u64,
) -> Result<(), PageError> {
  if page.total != total {
    return Err(PageError::Changed);
  }
  for row in page.items {
    let id = text(&row, "id");
    if id.is_empty() {
      return Err(PageError::Invalid);
    }
    if !seen.insert(id) {
      return Err(PageError::Changed);
    }
    rows.push(row);
  }
  Ok(())
}

async fn fetch_page(path: &str, page: u64) -> Result<Page, PageError> {
  let separator = if path.contains('?') { '&' } else { '?' };
  let path = format!("{path}{separator}page={page}&page_size={PAGE_SIZE}");
  parse_page(
    request("GET", &path, None)
      .await
      .map_err(PageError::Request)?,
    page,
  )
}

async fn fetch_collection(
  path: &str,
  cancelled: &impl Fn() -> bool,
) -> Result<Vec<Value>, PageError> {
  let first = fetch_page(path, 1).await?;
  let total = first.total;
  let last = total.div_ceil(PAGE_SIZE).max(1);
  let mut rows = Vec::new();
  let mut seen = BTreeSet::new();
  append_page(&mut rows, &mut seen, first, total)?;
  let pages = stream::iter(2..=last)
    .map(|page| fetch_page(path, page))
    .buffered(4);
  futures::pin_mut!(pages);
  while let Some(page) = pages.next().await {
    if cancelled() {
      return Err(PageError::Cancelled);
    }
    append_page(&mut rows, &mut seen, page?, total)?;
  }
  if rows.len() as u64 != total {
    return Err(PageError::Changed);
  }
  Ok(rows)
}

/// Load all public selector rows, with bounded I/O and no partial-list success.
pub async fn list_all_pages(path: &str, cancelled: impl Fn() -> bool) -> Result<Value, String> {
  for attempt in 0..2 {
    if cancelled() {
      return Err(String::new());
    }
    match fetch_collection(path, &cancelled).await {
      Ok(rows) => return Ok(Value::Array(rows)),
      Err(PageError::Changed) if attempt == 0 => continue,
      Err(PageError::Changed) => return Err(crate::i18n::t(CHANGED)),
      Err(PageError::Invalid) => return Err(crate::i18n::t(INVALID)),
      Err(PageError::Request(error)) => return Err(error),
      Err(PageError::Cancelled) => return Err(String::new()),
    }
  }
  unreachable!("bounded pagination retry always returns")
}

#[cfg(test)]
#[path = "../tests/pagination.rs"]
mod tests;
