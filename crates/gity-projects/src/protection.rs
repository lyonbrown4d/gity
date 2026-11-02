use crate::{ProtectionCommand, RepositoryAccessRepository};
use gity_core::AppError;
use gity_entities::{
  project_branch_protections as protections, project_deploy_keys as keys, projects, users,
};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, EntityTrait, ExprTrait, IntoActiveModel, QueryFilter, QueryOrder,
  QuerySelect, QueryTrait, Set, TransactionTrait, sea_query::Expr,
};
impl RepositoryAccessRepository {
  pub(crate) async fn protections(
    &self,
    project: i64,
  ) -> Result<Vec<protections::Model>, AppError> {
    Ok(
      protections::Entity::find()
        .filter(protections::Column::ProjectId.eq(project))
        .order_by_asc(protections::Column::BranchName)
        .all(&self.connection)
        .await?,
    )
  }
  pub(crate) async fn save_protection(
    &self,
    project: i64,
    branch: &str,
    command: &ProtectionCommand,
  ) -> Result<protections::Model, AppError> {
    let tx = self.connection.begin().await?;
    crate::lfs::lock_project(&tx, project).await?;
    let existing = protections::Entity::find()
      .filter(protections::Column::ProjectId.eq(project))
      .filter(protections::Column::BranchName.eq(branch))
      .one(&tx)
      .await?;
    let operation = if existing.is_some() {
      "update"
    } else {
      "insert"
    };
    let now = chrono::Utc::now().naive_utc();
    let mut active = existing
      .map(|row| row.into_active_model())
      .unwrap_or_else(|| protections::ActiveModel {
        id: Set(gity_core::new_id()),
        project_id: Set(project),
        branch_name: Set(branch.into()),
        created_at: Set(now),
        ..Default::default()
      });
    active.rule_type = Set(command.rule_type.clone());
    active.push_access_level = Set(command.push_access_level.clone());
    active.merge_access_level = Set(command.merge_access_level.clone());
    active.require_merge_request = Set(command.require_merge_request.into());
    active.require_pipeline_success = Set(command.require_pipeline_success.into());
    active.allow_force_push = Set(command.allow_force_push.into());
    active.allow_delete = Set(command.allow_delete.into());
    active.updated_at = Set(now);
    let row = if operation == "insert" {
      active.insert(&tx).await?
    } else {
      active.update(&tx).await?
    };
    gity_persistence::audit_protection(&tx, &row, operation, command.actor).await?;
    tx.commit().await?;
    Ok(row)
  }
  pub(crate) async fn delete_protection(
    &self,
    project: i64,
    branch: &str,
  ) -> Result<bool, AppError> {
    let tx = self.connection.begin().await?;
    crate::lfs::lock_project(&tx, project).await?;
    let row = protections::Entity::find()
      .filter(protections::Column::ProjectId.eq(project))
      .filter(protections::Column::BranchName.eq(branch))
      .one(&tx)
      .await?;
    let found = row.is_some();
    if let Some(row) = row {
      protections::Entity::delete_by_id(row.id).exec(&tx).await?;
      gity_persistence::audit_protection(&tx, &row, "delete", None).await?;
    }
    tx.commit().await?;
    Ok(found)
  }
  pub(crate) async fn authorized_keys(
    &self,
    fingerprint: &str,
  ) -> Result<Vec<keys::Model>, AppError> {
    let active_users = users::Entity::find()
      .select_only()
      .column(users::Column::Id)
      .filter(users::Column::Status.eq("active"))
      .into_query();
    let active_projects = projects::Entity::find()
      .select_only()
      .column(projects::Column::Id)
      .filter(projects::Column::Status.eq("active"))
      .filter(projects::Column::DeletedAt.is_null())
      .into_query();
    Ok(
      keys::Entity::find()
        .filter(keys::Column::Fingerprint.eq(fingerprint))
        .filter(Expr::col(keys::Column::CreatedByUserId).in_subquery(active_users))
        .filter(Expr::col(keys::Column::ProjectId).in_subquery(active_projects))
        .all(&self.connection)
        .await?,
    )
  }
  pub(crate) async fn touch_deploy_key(&self, key: i64) -> Result<(), AppError> {
    keys::ActiveModel {
      id: Set(key),
      last_used_at: Set(Some(chrono::Utc::now().naive_utc())),
      ..Default::default()
    }
    .update(&self.connection)
    .await?;
    Ok(())
  }
}
