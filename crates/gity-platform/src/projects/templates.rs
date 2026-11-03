use super::*;

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct LicenseTemplate {
  pub id: String,
  pub name: String,
  pub spdx_id: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct GitignoreTemplate {
  pub id: String,
  pub name: String,
}

#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(get,path="/repository-templates/licenses",tag="projects",
  description="Lists embedded license templates available when initializing a project. No runtime downloads.",
  responses((status=200,description="Available license templates",body=api::Envelope<Vec<LicenseTemplate>>),api::Errors),
  security(()))]
pub(super) async fn licenses() -> Reply {
  let templates = gity_projects::repository_templates::license_templates()
    .iter()
    .map(|template| LicenseTemplate {
      id: template.id.into(),
      name: template.name.into(),
      spdx_id: template.spdx_id.into(),
    })
    .collect::<Vec<_>>();
  Ok(ok(json!(templates)))
}

#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(get,path="/repository-templates/gitignores",tag="projects",
  description="Lists embedded github/gitignore presets available when initializing a project.",
  responses((status=200,description="Available gitignore templates",body=api::Envelope<Vec<GitignoreTemplate>>),api::Errors),
  security(()))]
pub(super) async fn gitignores() -> Reply {
  let templates = gity_projects::repository_templates::gitignore_templates()
    .iter()
    .map(|template| GitignoreTemplate {
      id: template.id.into(),
      name: template.name.into(),
    })
    .collect::<Vec<_>>();
  Ok(ok(json!(templates)))
}
