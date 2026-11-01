use super::*;
use std::collections::BTreeMap;
pub(crate) async fn environment(
    s: &AppState,
    project_id: i64,
    data: &Value,
) -> Result<(BTreeMap<String, String>, Vec<String>), AppError> {
    let project =
        s.db.one(
            "SELECT * FROM projects WHERE id=? AND deleted_at IS NULL",
            vec![project_id.into()],
        )
        .await?
        .ok_or_else(|| AppError::not_found("project not found"))?;
    let branch = text(data, "ref_name");
    let repo = crate::repository::repo(&project);
    let rules =
        s.db.all(
            "SELECT * FROM project_branch_protections WHERE project_id=?",
            vec![project_id.into()],
        )
        .await?;
    let protected = branch == text(&project, "default_branch")
        || crate::repository::effective_rule(&rules, branch).is_some();
    // A caller cannot obtain protected variables by naming main while running
    // the arbitrary object ID of a different, untrusted branch.
    let trusted = data["trusted_config"] == true
        && protected
        && s.git
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
    let key = s.db.quote_identifier("key")?;
    let rows =
        s.db.all(
            &format!("SELECT * FROM project_ci_variables WHERE project_id=? ORDER BY {key}"),
            vec![project_id.into()],
        )
        .await?;
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
