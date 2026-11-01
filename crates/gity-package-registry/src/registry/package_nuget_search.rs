use super::packages::named;
use super::*;
use crate::openapi::schemas as api;
use axum::extract::{Path, Query, State};
use serde::Deserialize;

#[derive(Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in=Query)]
pub(super) struct SearchQuery {
  q: Option<String>,
  skip: Option<u64>,
  take: Option<u64>,
}

#[utoipa::path(
    get,path="/projects/{id}/packages/nuget/query",tag="package protocols",
    params(api::ProjectPath,SearchQuery),
    responses((status=200,description="NuGet package search response"),api::Errors),
    security((),("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
pub(super) async fn search(
  State(state): State<AppState>,
  headers: HeaderMap,
  Path(project_id): Path<i64>,
  Query(query): Query<SearchQuery>,
) -> Result<Json<Value>, AppError> {
  authorize(&state, &headers, project_id, PackageAction::Read).await?;
  let take = query.take.unwrap_or(20).clamp(1, 100);
  let page = query.skip.unwrap_or(0) / take + 1;
  let (packages, total) = state
    .artifacts
    .repository
    .list_packages(
      project_id,
      Some(gity_artifacts::PackageFormat::Nuget),
      query.q.as_deref(),
      page,
      take,
    )
    .await?;
  let mut data = Vec::with_capacity(packages.len());
  for package in packages {
    let detail = named(&state, project_id, "nuget", &package.name).await?;
    let versions = detail["versions"]
      .as_array()
      .cloned()
      .unwrap_or_default()
      .into_iter()
      .filter(|version| text(version, "status") != "unlisted")
      .collect::<Vec<_>>();
    let latest = versions
      .iter()
      .filter_map(|v| v["version"].as_str())
      .max_by(|left, right| {
        semver::Version::parse(left)
          .ok()
          .cmp(&semver::Version::parse(right).ok())
      })
      .unwrap_or("");
    data.push(json!({
        "id":package.name,
        "version":latest,
        "versions":versions.iter().filter_map(|v| v["version"].as_str())
            .map(|version| json!({"version":version,"downloads":0})).collect::<Vec<_>>()
    }));
  }
  Ok(Json(json!({"totalHits":total,"data":data})))
}
