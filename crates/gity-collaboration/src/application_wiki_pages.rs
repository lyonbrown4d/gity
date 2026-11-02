use crate::{CollaborationService, domain::*};
use gity_core::AppError;
use gity_entities::project_wiki_pages as entity;

impl CollaborationService {
  pub async fn create_wiki_page(&self, command: WikiPageCreate) -> Result<entity::Model, AppError> {
    if self
      .repository
      .find_wiki_page(command.project_id, &command.slug)
      .await?
      .is_some()
    {
      return Err(AppError::conflict("wiki page already exists"));
    }
    if !nonempty(&command.title) {
      return Err(AppError::bad_request("title is required"));
    }
    self.repository.create_wiki_page(command).await
  }
  pub async fn update_wiki_page(
    &self,
    project: i64,
    key: &str,
    changes: WikiPageChanges,
  ) -> Result<entity::Model, AppError> {
    self
      .repository
      .update_wiki_page(project, key, changes)
      .await
  }
  pub async fn wiki_page(&self, project: i64, slug: &str) -> Result<entity::Model, AppError> {
    self.repository.wiki_page(project, slug).await
  }
  pub async fn find_wiki_page(
    &self,
    project: i64,
    slug: &str,
  ) -> Result<Option<entity::Model>, AppError> {
    self.repository.find_wiki_page(project, slug).await
  }
  pub async fn wiki_pages(&self, project: i64) -> Result<Vec<entity::Model>, AppError> {
    self.repository.wiki_pages(project).await
  }
  pub async fn delete_wiki_page(
    &self,
    project: i64,
    slug: &str,
  ) -> Result<entity::Model, AppError> {
    self.repository.delete_wiki_page(project, slug).await
  }
}
