use gity_core::AppError;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, Condition, DatabaseTransaction, EntityTrait, IntoActiveModel,
  QueryFilter, Set,
};

pub(crate) async fn rename(
  tx: &DatabaseTransaction,
  old: &str,
  next: &str,
) -> Result<(), AppError> {
  let prefix = format!("{old}/");
  let organizations = gity_entities::organizations::Entity::find()
    .filter(
      Condition::any()
        .add(gity_entities::organizations::Column::FullPath.eq(old))
        .add(gity_entities::organizations::Column::FullPath.starts_with(&prefix)),
    )
    .all(tx)
    .await?
    .into_iter()
    .filter(|row| row.full_path == old || row.full_path.starts_with(&prefix))
    .collect::<Vec<_>>();
  let projects = gity_entities::projects::Entity::find()
    .filter(
      Condition::any()
        .add(gity_entities::projects::Column::FullPath.eq(old))
        .add(gity_entities::projects::Column::FullPath.starts_with(&prefix)),
    )
    .all(tx)
    .await?
    .into_iter()
    .filter(|row| row.full_path == old || row.full_path.starts_with(&prefix))
    .collect::<Vec<_>>();
  let timestamp = chrono::Utc::now().naive_utc();
  for row in organizations {
    let suffix = row
      .full_path
      .strip_prefix(old)
      .unwrap_or_default()
      .to_owned();
    let mut active = row.into_active_model();
    active.full_path = Set(format!("{next}{suffix}"));
    active.updated_at = Set(timestamp);
    active.update(tx).await?;
  }
  for row in projects {
    let suffix = row
      .full_path
      .strip_prefix(old)
      .unwrap_or_default()
      .to_owned();
    let mut active = row.into_active_model();
    active.full_path = Set(format!("{next}{suffix}"));
    active.updated_at = Set(timestamp);
    active.update(tx).await?;
  }
  Ok(())
}
