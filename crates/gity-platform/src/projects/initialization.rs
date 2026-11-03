//! Typed initial-file selection validated before a project is persisted.
use gity_core::AppError;
use serde::Deserialize;
use serde_json::Value;

#[derive(Default, Deserialize)]
pub(super) struct Initialization {
  license_template: Option<String>,
  license_copyright_holder: Option<String>,
  gitignore_template: Option<String>,
  #[serde(default)]
  initialize_with_readme: Option<bool>,
}
impl Initialization {
  pub fn parse(body: &Value) -> Result<Self, AppError> {
    serde_json::from_value(body.clone())
      .map_err(|_| AppError::bad_request("invalid repository initialization options"))
  }

  pub fn files(
    &self,
    name: &str,
    description: Option<&str>,
    owner: &str,
  ) -> Result<Vec<gity_git::InitialFile>, AppError> {
    use chrono::Datelike;
    use gity_projects::repository_templates::{RepositoryTemplateOptions, template_files};
    let holder = self
      .license_copyright_holder
      .as_deref()
      .map(str::trim)
      .filter(|s| !s.is_empty())
      .unwrap_or(owner);
    let files = template_files(RepositoryTemplateOptions {
      name,
      description,
      license: self.license_template.as_deref(),
      gitignore: self.gitignore_template.as_deref(),
      readme: self.initialize_with_readme.unwrap_or(false),
      year: chrono::Utc::now().year(),
      copyright_holder: holder,
    })?;
    Ok(
      files
        .into_iter()
        .map(|f| gity_git::InitialFile {
          path: f.path,
          content: f.content,
        })
        .collect(),
    )
  }
}
