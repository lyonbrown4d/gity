use crate::{
  AppState,
  access::{self, ProjectAction},
  openapi::schemas as api,
};
use axum::{
  Json,
  extract::{Path, State},
  http::HeaderMap,
};
use gity_core::AppError;
use gity_entities::project_registry_settings as settings;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RegistryMode {
  Hosted,
  Mirror,
}
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct RegistryPolicy {
  pub mode: RegistryMode,
  pub mirror_alias: Option<String>,
  pub mirror_repository_prefix: Option<String>,
}
pub fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new().routes(utoipa_axum::routes!(get, put))
}
#[utoipa::path(get,path="/projects/{id}/registry/settings",tag="container registry",params(api::ProjectPath),
  responses((status=200,body=api::Envelope<RegistryPolicy>,description="Project container registry policy"),api::Errors),security(("bearer"=[])))]
async fn get(
  State(state): State<AppState>,
  headers: HeaderMap,
  Path(project): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
  access::project_action(&state, &headers, project, ProjectAction::RegistryAdmin).await?;
  let row = settings::Entity::find_by_id(project)
    .one(&state.db.connection)
    .await?;
  let policy = row
    .map(|row| RegistryPolicy {
      mode: if row.mode == "mirror" {
        RegistryMode::Mirror
      } else {
        RegistryMode::Hosted
      },
      mirror_alias: row.mirror_alias,
      mirror_repository_prefix: Some(row.mirror_repository_prefix),
    })
    .unwrap_or(RegistryPolicy {
      mode: RegistryMode::Hosted,
      mirror_alias: None,
      mirror_repository_prefix: None,
    });
  Ok(gity_core::ok(serde_json::to_value(policy).map_err(
    |_| AppError::internal("serialize registry policy"),
  )?))
}
#[utoipa::path(put,path="/projects/{id}/registry/settings",tag="container registry",params(api::ProjectPath),request_body=RegistryPolicy,
  responses((status=200,body=api::Envelope<RegistryPolicy>,description="Update hosted or read-only mirror policy"),api::Errors),security(("bearer"=[])))]
async fn put(
  State(state): State<AppState>,
  headers: HeaderMap,
  Path(project): Path<i64>,
  Json(policy): Json<RegistryPolicy>,
) -> Result<Json<serde_json::Value>, AppError> {
  let actor = access::project_action(&state, &headers, project, ProjectAction::RegistryAdmin)
    .await?
    .actor()?;
  let prefix = policy
    .mirror_repository_prefix
    .as_deref()
    .unwrap_or("")
    .trim_matches('/');
  if prefix.len() > 255
    || prefix
      .split('/')
      .any(|part| matches!(part, "." | "..") || part.contains(['\\', '\0']))
  {
    return Err(AppError::bad_request("invalid mirror repository prefix"));
  }
  let alias = match policy.mode {
    RegistryMode::Hosted => None,
    RegistryMode::Mirror => {
      let alias = policy
        .mirror_alias
        .as_deref()
        .ok_or_else(|| AppError::bad_request("mirror alias is required"))?;
      let configured = state
        .settings
        .components
        .registry
        .mirrors
        .get(alias)
        .ok_or_else(|| AppError::bad_request("unknown mirror alias"))?;
      if !configured.allows_project(project) {
        return Err(AppError::forbidden(
          "mirror source is not delegated to this project",
        ));
      }
      Some(alias.to_owned())
    }
  };
  let existing = settings::Entity::find_by_id(project)
    .one(&state.db.connection)
    .await?;
  let is_new = existing.is_none();
  let mut row = existing
    .map(IntoActiveModel::into_active_model)
    .unwrap_or_else(|| settings::ActiveModel {
      project_id: Set(project),
      ..Default::default()
    });
  row.mode = Set(
    match policy.mode {
      RegistryMode::Hosted => "hosted",
      RegistryMode::Mirror => "mirror",
    }
    .into(),
  );
  row.mirror_alias = Set(alias);
  row.mirror_repository_prefix = Set(prefix.to_owned());
  row.updated_by = Set(Some(actor.user_id));
  row.updated_at = Set(chrono::Utc::now().naive_utc());
  if is_new {
    row.insert(&state.db.connection).await?;
  } else {
    row.update(&state.db.connection).await?;
  }
  Ok(gity_core::ok(serde_json::to_value(policy).map_err(
    |_| AppError::internal("serialize registry policy"),
  )?))
}
