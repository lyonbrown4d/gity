use super::*;
use std::collections::BTreeMap;
pub(crate) async fn environment(
  s: &AppState,
  project_id: i64,
  data: &Value,
) -> Result<(BTreeMap<String, String>, Vec<String>), AppError> {
  let project = model_value(s.ci.project(project_id).await?);
  let branch = text(data, "ref_name");
  let repo = crate::repository::repo(&project);
  let rules = s
    .ci
    .protections(project_id)
    .await?
    .into_iter()
    .map(model_value)
    .collect::<Vec<_>>();
  let protected = branch == text(&project, "default_branch")
    || crate::repository::effective_rule(&rules, branch).is_some();
  // A caller cannot obtain protected variables by naming main while running
  // the arbitrary object ID of a different, untrusted branch.
  let trusted = data["trusted_config"] == true
    && protected
    && s
      .git
      .resolve_revision(&repo, branch, "")
      .await
      .map_err(git_error)?
      == text(data, "commit_sha");
  let mut env = data["yaml_env"]
    .as_object()
    .map(|m| {
      m.iter()
        .filter_map(|(k, v)| v.as_str().map(|v| (k.clone(), v.to_owned())))
        .collect::<BTreeMap<_, _>>()
    })
    .unwrap_or_default();
  let rows = s
    .ci
    .variables(project_id)
    .await?
    .into_iter()
    .map(model_value)
    .collect::<Vec<_>>();
  let mut masked = Vec::new();
  for row in rows {
    if flag(&row, "protected") && !trusted {
      continue;
    }
    let value = text(&row, "value").to_owned();
    if flag(&row, "masked") && !value.is_empty() {
      masked.push(value.clone());
    }
    env.insert(text(&row, "key").into(), value);
  }
  for (key, value) in [
    ("CI", "true"),
    ("CI_PROJECT_PATH", text(&project, "full_path")),
    ("CI_COMMIT_SHA", text(data, "commit_sha")),
    ("CI_COMMIT_REF_NAME", branch),
    ("CI_JOB_NAME", text(data, "pipeline_job_name")),
  ] {
    env.insert(key.into(), value.into());
  }
  Ok((env, masked))
}
