use crate::composition::Runtime;
use anyhow::{Context, Result, bail};
use serde_json::Value;
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
            let rows=runtime.db.all("SELECT * FROM projects WHERE deleted_at IS NULL AND id>? ORDER BY id LIMIT 100",vec![after.into()]).await?;
            if rows.is_empty() {
                break;
            }
            for project in rows {
                after = id(&project)?;
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
            let sql = format!(
                "SELECT * FROM projects WHERE deleted_at IS NULL AND id IN ({}) ORDER BY id",
                vec!["?"; chunk.len()].join(",")
            );
            let rows = runtime
                .db
                .all(&sql, chunk.iter().map(|id| (*id).into()).collect())
                .await?;
            if rows.len() != chunk.len() {
                bail!("one or more selected projects do not exist or were deleted");
            }
            for project in rows {
                count += 1;
                if let Err(error) = refresh(runtime, &project).await {
                    failures += 1;
                    eprintln!("project {}: {error:#}", id(&project)?);
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
async fn refresh(runtime: &Runtime, project: &Value) -> Result<()> {
    let repo = format!(
        "{}.git",
        project["full_path"]
            .as_str()
            .context("project path missing")?
    );
    let branch = project["default_branch"]
        .as_str()
        .context("project default branch missing")?;
    match runtime.search.refresh(id(project)?, &repo, branch).await {
        Ok(status) => {
            println!(
                "Project {} revision {}: {} files, {} bytes{}",
                id(project)?,
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
fn id(project: &Value) -> Result<i64> {
    project["id"].as_i64().context("invalid project identifier")
}
