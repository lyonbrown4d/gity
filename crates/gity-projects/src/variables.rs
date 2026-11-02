use crate::{RepositoryAccessRepository, VariableCommand};
use gity_core::AppError;
use gity_entities::project_ci_variables as variables;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, Set,
  TransactionTrait,
};
impl RepositoryAccessRepository {
  pub(crate) async fn variables(&self, project: i64) -> Result<Vec<variables::Model>, AppError> {
    Ok(
      variables::Entity::find()
        .filter(variables::Column::ProjectId.eq(project))
        .order_by_asc(variables::Column::Key)
        .all(&self.connection)
        .await?,
    )
  }
  pub(crate) async fn save_variable(
    &self,
    project: i64,
    command: VariableCommand,
  ) -> Result<variables::Model, AppError> {
    let tx = self.connection.begin().await?;
    // Serialize writers through the project before the scoped lookup and snapshot.
    crate::lfs::lock_project(&tx, project).await?;
    let existing = variables::Entity::find()
      .filter(variables::Column::ProjectId.eq(project))
      .filter(variables::Column::Key.eq(&command.key))
      .one(&tx)
      .await?;
    let now = chrono::Utc::now().naive_utc();
    let (row, operation) = if let Some(existing) = existing {
      let mut active = existing.into_active_model();
      active.value = Set(command.value);
      active.masked = Set(command.masked.into());
      active.protected = Set(command.protected.into());
      active.updated_at = Set(now);
      (active.update(&tx).await?, "update")
    } else {
      (
        variables::ActiveModel {
          id: Set(gity_core::new_id()),
          project_id: Set(project),
          key: Set(command.key),
          value: Set(command.value),
          masked: Set(command.masked.into()),
          protected: Set(command.protected.into()),
          created_at: Set(now),
          updated_at: Set(now),
        }
        .insert(&tx)
        .await?,
        "insert",
      )
    };
    gity_persistence::audit_variable(&tx, &row, operation, None).await?;
    tx.commit().await?;
    Ok(row)
  }
  pub(crate) async fn delete_variable(&self, project: i64, key: &str) -> Result<(), AppError> {
    let tx = self.connection.begin().await?;
    crate::lfs::lock_project(&tx, project).await?;
    if let Some(row) = variables::Entity::find()
      .filter(variables::Column::ProjectId.eq(project))
      .filter(variables::Column::Key.eq(key))
      .one(&tx)
      .await?
    {
      variables::Entity::delete_by_id(row.id).exec(&tx).await?;
      gity_persistence::audit_variable(&tx, &row, "delete", None).await?;
    }
    tx.commit().await?;
    Ok(())
  }
}
