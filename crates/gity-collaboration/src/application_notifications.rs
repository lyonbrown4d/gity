use crate::{CollaborationService, domain::Notification};
use gity_core::AppError;
use gity_entities::project_todos as todos;

impl CollaborationService {
  pub async fn todos(
    &self,
    user: i64,
    state: &str,
    limit: u64,
  ) -> Result<Vec<todos::Model>, AppError> {
    self.repository.todos(user, state, limit).await
  }
  pub async fn complete_todo(&self, user: i64, id: i64) -> Result<(), AppError> {
    self.repository.complete_todo(user, id).await
  }
  pub async fn notify(&self, command: Notification) -> Result<(), AppError> {
    self.repository.notify(command).await
  }
}
