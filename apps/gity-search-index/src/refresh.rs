use crate::composition::Runtime;
use anyhow::{Result, bail};
use gity_entities::projects::Model;
use std::collections::BTreeSet;
pub async fn run(runtime: &Runtime, selected: &[i64]) -> Result<()> {
  if selected.iter().any(|id| *id <= 0) {
    bail!("project IDs must be positive");
  }
  let mut count = 0;
  let mut failures = 0;
  if selected.is_empty() {
    let mut after = 0i64;
    loop {
      let rows = runtime.projects.search_projects_after(after, 100).await?;
      if rows.is_empty() {
        break;
      }
      for project in rows {
        after = project.id;
        count += 1;
        if let Err(error) = refresh(runtime, &project).await {
          failures += 1;
          eprintln!("project {after}: {error:#}");
        }
      }
    }
  } else {
    let selected = selected
      .iter()
      .copied()
      .collect::<BTreeSet<_>>()
      .into_iter()
      .collect::<Vec<_>>();
    for chunk in selected.chunks(100) {
      let rows = runtime.projects.search_projects_selected(chunk).await?;
      if rows.len() != chunk.len() {
        bail!("one or more selected projects do not exist or were deleted");
      }
      for project in rows {
        count += 1;
        if let Err(error) = refresh(runtime, &project).await {
          failures += 1;
          eprintln!("project {}: {error:#}", project.id);
        }
      }
    }
  }
  if failures > 0 {
    bail!("{failures} of {count} project indexes could not be refreshed");
  }
  println!("Refreshed {count} project indexes");
  Ok(())
}
async fn refresh(runtime: &Runtime, project: &Model) -> Result<()> {
  let repo = format!("{}.git", project.full_path);
  let branch = &project.default_branch;
  match runtime.search.refresh(project.id, &repo, branch).await {
    Ok(status) => {
      println!(
        "Project {} revision {}: {} files, {} bytes{}",
        project.id,
        status.revision,
        status.indexed_files,
        status.total_bytes,
        if status.unchanged { " (unchanged)" } else { "" }
      );
      Ok(())
    }
    Err(error) => Err(error),
  }
}
