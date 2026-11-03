use crate::AppState;
use gity_core::AppError;
use gity_entities::projects;
use gity_search::global::{ResourceKind, SearchDocument};
use sha2::{Digest, Sha256};

pub async fn build(s: &AppState, p: &projects::Model) -> Result<Vec<SearchDocument>, AppError> {
  let mut documents = Vec::new();
  let mut after = 0;
  let mut bytes = 0usize;
  loop {
    let rows = s
      .collaboration
      .search_issues_after(p.id, after, 200)
      .await?;
    if rows.is_empty() {
      break;
    }
    for row in rows {
      after = row.id;
      bytes =
        bytes.saturating_add(row.title.len() + row.description.as_ref().map_or(0, String::len));
      if bytes > 256 * 1024 * 1024 {
        return Err(AppError::conflict("project search byte budget exceeded"));
      }
      if documents.len() >= 100_000 {
        return Err(AppError::conflict("project issue search budget exceeded"));
      }
      documents.push(SearchDocument {
        key: format!("issue:{}:{}", p.id, row.id),
        project_id: p.id,
        kind: ResourceKind::Issue,
        resource_id: row.id.to_string(),
        version: row.updated_at.to_string(),
        title: row.title,
        body: row.description.unwrap_or_default(),
        path: String::new(),
        iid: Some(row.iid),
        line_start: 0,
        confidential: row.confidential != 0,
      });
    }
  }
  let repo = format!("{}.git", p.id);
  let branches = s
    .git
    .list_branches(&repo, &p.default_branch)
    .await
    .map_err(source)?;
  if branches.as_array().is_some_and(|v| v.is_empty()) {
    return Ok(documents);
  }
  let sha = s
    .git
    .resolve_revision(&repo, &p.default_branch, "")
    .await
    .map_err(source)?;
  let mut pending = vec![String::new()];
  let mut files = 0;
  let mut visited = 0;
  while let Some(path) = pending.pop() {
    let tree = s
      .git
      .list_tree(&repo, &sha, "", &path)
      .await
      .map_err(source)?;
    for entry in tree
      .as_array()
      .ok_or_else(|| AppError::internal("invalid Git tree response"))?
    {
      visited += 1;
      if visited > 20_000 {
        return Err(AppError::conflict("project search tree budget exceeded"));
      }
      let path = entry["path"]
        .as_str()
        .ok_or_else(|| AppError::internal("invalid Git path"))?
        .to_owned();
      if entry["type"] == "tree" {
        pending.push(path);
        continue;
      }
      if entry["type"] != "blob"
        || entry["size"].as_u64().unwrap_or(u64::MAX) > s.settings.search.max_file_size_bytes as u64
      {
        continue;
      }
      files += 1;
      if files > s.settings.search.max_files_per_project {
        return Err(AppError::conflict(
          "project code search file budget exceeded",
        ));
      }
      let blob = s
        .git
        .get_blob_limited(
          &repo,
          &sha,
          "",
          &path,
          s.settings.search.max_file_size_bytes,
        )
        .await
        .map_err(source)?;
      if blob["encoding"] != "utf-8" {
        continue;
      }
      let body = blob["content"].as_str().unwrap_or_default();
      if body.contains('\0') {
        continue;
      }
      bytes = bytes.saturating_add(body.len());
      if bytes > 256 * 1024 * 1024 {
        return Err(AppError::conflict(
          "project code search byte budget exceeded",
        ));
      }
      documents.push(SearchDocument {
        key: format!("code:{}:{path}", p.id),
        project_id: p.id,
        kind: ResourceKind::Code,
        resource_id: hex::encode(Sha256::digest(body.as_bytes())),
        version: sha.clone(),
        title: path.clone(),
        body: body.to_owned(),
        path,
        iid: None,
        line_start: 1,
        confidential: false,
      });
    }
  }
  Ok(documents)
}
fn source(_: anyhow::Error) -> AppError {
  AppError::new(
    axum::http::StatusCode::SERVICE_UNAVAILABLE,
    "Git indexing source unavailable",
  )
}
