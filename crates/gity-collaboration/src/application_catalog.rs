use crate::{CollaborationService, domain::*};
use gity_core::AppError;
use gity_entities::project_labels as labels;

impl CollaborationService {
  pub async fn labels(&self, project: i64) -> Result<Vec<labels::Model>, AppError> {
    self.repository.labels(project).await
  }
  pub async fn label(&self, project: i64, id: i64) -> Result<labels::Model, AppError> {
    self.repository.label(project, id).await
  }
  pub async fn named_label(
    &self,
    project: i64,
    title: &str,
  ) -> Result<Option<labels::Model>, AppError> {
    self.repository.named_label(project, title).await
  }
  pub async fn create_label(
    &self,
    project: i64,
    command: LabelCreate,
  ) -> Result<labels::Model, AppError> {
    self.repository.create_label(project, command).await
  }
  pub async fn update_label(
    &self,
    project: i64,
    id: i64,
    command: LabelChanges,
  ) -> Result<labels::Model, AppError> {
    self.repository.update_label(project, id, command).await
  }
  pub async fn delete_label(&self, project: i64, id: i64) -> Result<(), AppError> {
    self.repository.delete_label(project, id).await
  }
  pub async fn issue_label_rows(
    &self,
    project: i64,
    issues: Vec<i64>,
  ) -> Result<Vec<(i64, labels::Model)>, AppError> {
    self.repository.issue_label_rows(project, issues).await
  }
  pub async fn replace_labels(
    &self,
    project: i64,
    iid: i64,
    wanted: Vec<(String, Option<String>)>,
  ) -> Result<(), AppError> {
    self.repository.replace_labels(project, iid, wanted).await
  }
}
