use super::*;
pub(super) async fn query(
    s: &AppState,
    project: &Value,
    revision: &str,
    q: &HashMap<String, String>,
) -> Result<Value, AppError> {
    let params = gity_git::SearchParams {
        query: q
            .get("query")
            .or_else(|| q.get("q"))
            .cloned()
            .unwrap_or_default(),
        path: q.get("path").cloned().unwrap_or_default(),
        match_case: q.get("match_case").is_some_and(|v| v == "true"),
        regex: q.get("regex").is_some_and(|v| v == "true"),
        ..Default::default()
    };
    if s.settings.search.index_enabled {
        match s
            .search
            .query(
                projects::id(&project["id"])?,
                &repo(project),
                revision,
                default_branch(project),
                params.clone(),
            )
            .await
        {
            Ok(Some(result)) => return Ok(result),
            Ok(None) => {}
            Err(error) => {
                tracing::warn!(error=%error,project_id=%project["id"],"search index unavailable; scanning Git revision")
            }
        }
    }
    s.git
        .search(&repo(project), revision, default_branch(project), params)
        .await
        .map_err(git_error)
}
#[utoipa::path(post,path="/{id}/search/index/refresh",tag="Repository",params(crate::openapi::schemas::ProjectPath),
    responses((status=200,description="Rebuild bounded search index; maintainer required;409 when indexing disabled",body=crate::openapi::schemas::Envelope<crate::openapi::repository_schemas::IndexStatus>),crate::openapi::schemas::Errors),security(("bearer"=[]),("private_token"=[])))]
pub(super) async fn refresh(State(s): State<AppState>, h: HeaderMap, Path(id): Path<i64>) -> Reply {
    let project = access::project_action(&s, &h, id, access::ProjectAction::RepositoryAdmin)
        .await?
        .project;
    if !s.settings.search.index_enabled {
        return Err(AppError::conflict("search indexing is disabled"));
    }
    let status = s
        .search
        .refresh(id, &repo(&project), default_branch(&project))
        .await
        .map_err(git_error)?;
    Ok(ok(json!(status)))
}
