use crate::{
  AppState,
  access::{self, ProjectAction},
};
use axum::http::{HeaderMap, StatusCode};
use gity_core::AppError;
use gity_oci::{
  AuthorizationError, MirrorSource, OciAuthorization, OciAuthorizationDecision, OciOperation,
  OciRequest,
};

#[derive(Clone)]
pub struct PlatformOciAuthorizer {
  state: AppState,
}
impl PlatformOciAuthorizer {
  pub fn new(state: AppState) -> Self {
    Self { state }
  }
}

#[async_trait::async_trait]
impl OciAuthorization for PlatformOciAuthorizer {
  async fn authorize(
    &self,
    request: &OciRequest,
    authorization: Option<&str>,
  ) -> Result<OciAuthorizationDecision, AuthorizationError> {
    if request.operation == OciOperation::Catalog {
      return Err(AuthorizationError::NotFound);
    }
    let mut headers = HeaderMap::new();
    if let Some(value) = authorization {
      headers.insert(
        "authorization",
        value
          .parse()
          .map_err(|_| AuthorizationError::Unauthenticated)?,
      );
    }
    if request.repository.is_empty() {
      self.probe(&headers).await.map_err(map_error)?;
      return Ok(OciAuthorizationDecision {
        storage_repository: String::new(),
        mount_source_storage_repository: None,
        mirror_source: None,
      });
    }
    let target = self
      .resolve(
        &request.repository,
        request.operation,
        &headers,
        request.reference.clone(),
      )
      .await
      .map_err(map_error)?;
    let mount_source_storage_repository = if let Some(source) = &request.mount_source {
      Some(
        self
          .resolve(source, OciOperation::Pull, &headers, None)
          .await
          .map_err(map_error)?
          .storage_repository,
      )
    } else {
      None
    };
    Ok(OciAuthorizationDecision {
      mount_source_storage_repository,
      ..target
    })
  }
}
impl PlatformOciAuthorizer {
  async fn probe(&self, headers: &HeaderMap) -> Result<(), AppError> {
    let Some(raw) = crate::registry::token_from_headers(headers) else {
      if headers.contains_key("authorization") {
        return Err(AppError::unauthorized("unsupported registry credentials"));
      }
      return Ok(());
    };
    if raw.starts_with("gity_job_v1.") {
      crate::ci::authenticate_job_credential(&self.state, &raw).await?;
      return Ok(());
    }
    if let Some(token) = crate::registry::authenticate_token(&self.state, &raw).await? {
      if !token.allows("read_registry") && !token.allows("write_registry") {
        return Err(AppError::forbidden("token lacks container registry scope"));
      }
      return Ok(());
    }
    crate::auth::principal_token(&self.state, &raw).await?;
    Ok(())
  }
  async fn resolve(
    &self,
    repository: &str,
    operation: OciOperation,
    headers: &HeaderMap,
    reference: Option<String>,
  ) -> Result<OciAuthorizationDecision, AppError> {
    if repository.len() > 255 || repository.split('/').count() > 32 {
      return Err(AppError::not_found("repository not found"));
    }
    let segments = repository.split('/').collect::<Vec<_>>();
    let candidates = (2..=segments.len())
      .map(|count| segments[..count].join("/"))
      .collect::<Vec<_>>();
    let project = self
      .state
      .projects
      .paths(candidates)
      .await?
      .into_iter()
      .max_by_key(|project| project.full_path.len())
      .ok_or_else(|| AppError::not_found("repository not found"))?;
    let write = matches!(operation, OciOperation::Push | OciOperation::Delete);
    let job = crate::ci::package_identity(&self.state, headers, project.id, write).await?;
    if job.is_some() && operation == OciOperation::Delete {
      return Err(AppError::forbidden("CI job tokens cannot delete images"));
    }
    if job.is_none() {
      let action = match operation {
        OciOperation::Pull => ProjectAction::ContainerRead,
        OciOperation::Push => ProjectAction::ContainerWrite,
        OciOperation::Delete => ProjectAction::ContainerDelete,
        OciOperation::Catalog | OciOperation::Probe => {
          return Err(AppError::forbidden("catalog disabled"));
        }
      };
      access::project_action(&self.state, headers, project.id, action).await?;
    }
    let suffix = repository
      .strip_prefix(&project.full_path)
      .unwrap()
      .strip_prefix('/')
      .unwrap_or("");
    let policy = self
      .state
      .repository_access
      .registry_settings(project.id)
      .await?;
    if let Some(policy) = policy
      && policy.mode == "mirror"
    {
      if write {
        return Err(AppError::forbidden("mirror repositories are read only"));
      }
      let alias = policy
        .mirror_alias
        .ok_or_else(|| AppError::internal("mirror alias is missing"))?;
      let config = self
        .state
        .settings
        .components
        .registry
        .mirrors
        .get(&alias)
        .ok_or_else(|| AppError::forbidden("mirror source is unavailable"))?;
      if !config.allows_project(project.id) {
        return Err(AppError::forbidden(
          "mirror source is not delegated to this project",
        ));
      }
      let upstream_repository = [policy.mirror_repository_prefix.trim_matches('/'), suffix]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("/");
      if upstream_repository.is_empty() {
        return Err(AppError::not_found("mirror image name is required"));
      }
      return Ok(OciAuthorizationDecision {
        storage_repository: format!("mirror-{alias}/{upstream_repository}"),
        mount_source_storage_repository: None,
        mirror_source: Some(MirrorSource {
          registry_alias: alias,
          repository: upstream_repository,
          reference,
        }),
      });
    }
    let storage_repository = if suffix.is_empty() {
      format!("projects/{}/root", project.id)
    } else {
      format!("projects/{}/repositories/{suffix}", project.id)
    };
    Ok(OciAuthorizationDecision {
      storage_repository,
      mount_source_storage_repository: None,
      mirror_source: None,
    })
  }
}
fn map_error(error: AppError) -> AuthorizationError {
  match error.status {
    StatusCode::UNAUTHORIZED => AuthorizationError::Unauthenticated,
    StatusCode::FORBIDDEN => AuthorizationError::Forbidden,
    StatusCode::NOT_FOUND => AuthorizationError::NotFound,
    _ => AuthorizationError::Unavailable,
  }
}
