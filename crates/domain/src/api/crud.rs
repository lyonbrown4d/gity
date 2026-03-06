use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Default, Deserialize, Serialize, IntoParams, ToSchema)]
pub struct IdsQuery {
  pub ids: Option<String>,
}

pub fn parse_csv_ids(raw: Option<&str>) -> Vec<String> {
  let mut dedup = HashSet::new();
  let mut ids = Vec::new();

  let Some(raw_ids) = raw else {
    return ids;
  };

  for item in raw_ids.split(',') {
    let id = item.trim();
    if id.is_empty() {
      continue;
    }
    if dedup.insert(id.to_string()) {
      ids.push(id.to_string());
    }
  }

  ids
}
