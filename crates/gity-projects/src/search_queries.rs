use crate::ProjectRepository;
use gity_core::AppError;
use gity_entities::projects;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

impl ProjectRepository {
  pub(crate) async fn search_projects_after(
    &self,
    cursor: i64,
    limit: u64,
  ) -> Result<Vec<projects::Model>, AppError> {
    Ok(
      projects::Entity::find()
        .filter(projects::Column::DeletedAt.is_null())
        .filter(projects::Column::Id.gt(cursor))
        .order_by_asc(projects::Column::Id)
        .limit(limit)
        .all(&self.connection)
        .await?,
    )
  }
  pub(crate) async fn search_projects_selected(
    &self,
    ids: &[i64],
  ) -> Result<Vec<projects::Model>, AppError> {
    Ok(
      projects::Entity::find()
        .filter(projects::Column::DeletedAt.is_null())
        .filter(projects::Column::Id.is_in(ids.to_vec()))
        .order_by_asc(projects::Column::Id)
        .all(&self.connection)
        .await?,
    )
  }
}
