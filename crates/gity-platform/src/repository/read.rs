use super::*;
pub(super) async fn read(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((id, action)): Path<(i64, String)>,
  Query(q): Query<HashMap<String, String>>,
) -> Reply {
  let project = access::project_action(&s, &h, id, access::ProjectAction::RepositoryRead)
    .await?
    .project;
  let path = repo(&project);
  let default = default_branch(&project);
  let revision = q
    .get("ref_name")
    .or_else(|| q.get("ref"))
    .or_else(|| q.get("branch_name"))
    .map(String::as_str)
    .unwrap_or("");
  let file = q.get("path").map(String::as_str).unwrap_or("");
  let value = match action.as_str() {
    "branches" => {
      let mut branches = s
        .git
        .list_branches(&path, default)
        .await
        .map_err(git_error)?;
      let rules = s
        .db
        .all(
          "SELECT * FROM project_branch_protections WHERE project_id=? ORDER BY branch_name",
          vec![id.into()],
        )
        .await?;
      if let Some(items) = branches.as_array_mut() {
        for branch in items {
          let rule = effective_rule(&rules, branch["name"].as_str().unwrap_or_default());
          branch["protected"] = json!(rule.is_some());
          branch["protection"] = rule.cloned().unwrap_or(Value::Null);
        }
      }
      return Ok(ok(views::present("branches", id, revision, branches)));
    }
    "tags" => s.git.list_tags(&path).await,
    "tree" => s.git.list_tree(&path, revision, default, file).await,
    "blob" => s.git.get_blob(&path, revision, default, file).await,
    "readme" => s.git.get_readme(&path, revision, default).await,
    "languages" => s.git.analyze_languages(&path, revision, default).await,
    "commits" => {
      s.git
        .list_commits(
          &path,
          revision,
          default,
          q.get("limit")
            .and_then(|v| v.parse().ok())
            .unwrap_or(50)
            .clamp(1, 500),
        )
        .await
    }
    "search" => return Ok(ok(search::query(&s, &project, revision, &q).await?)),
    "branch-protections" => {
      return Ok(ok(
        s.db
          .all(
            "SELECT * FROM project_branch_protections WHERE project_id=? ORDER BY branch_name",
            vec![id.into()],
          )
          .await?
          .into_iter()
          .map(views::protection)
          .collect::<Vec<_>>(),
      ));
    }
    _ => return Err(AppError::not_found("repository operation not found")),
  }
  .map_err(git_error)?;
  Ok(ok(views::present(
    &action,
    id,
    if revision.is_empty() {
      default
    } else {
      revision
    },
    value,
  )))
}
