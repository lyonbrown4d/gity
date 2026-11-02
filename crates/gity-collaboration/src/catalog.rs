use crate::{domain::*, repository::CollaborationRepository};
use gity_core::AppError;
use gity_entities::{project_label_links as links, project_labels as labels};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter,
  QueryOrder, Set, TransactionTrait,
};
const DEFAULT_COLOR: &str = "#428BCA";
impl CollaborationRepository {
  pub async fn labels(&self, project: i64) -> Result<Vec<labels::Model>, AppError> {
    Ok(
      labels::Entity::find()
        .filter(labels::Column::ProjectId.eq(project))
        .order_by_asc(labels::Column::Title)
        .order_by_asc(labels::Column::Id)
        .all(&self.connection)
        .await?,
    )
  }
  pub async fn label(&self, project: i64, id: i64) -> Result<labels::Model, AppError> {
    labels::Entity::find_by_id(id)
      .filter(labels::Column::ProjectId.eq(project))
      .one(&self.connection)
      .await?
      .ok_or_else(|| AppError::not_found("label not found"))
  }
  pub async fn named_label(
    &self,
    project: i64,
    title: &str,
  ) -> Result<Option<labels::Model>, AppError> {
    Ok(
      labels::Entity::find()
        .filter(labels::Column::ProjectId.eq(project))
        .filter(labels::Column::Title.eq(title))
        .one(&self.connection)
        .await?,
    )
  }
  pub async fn create_label(
    &self,
    project: i64,
    command: LabelCreate,
  ) -> Result<labels::Model, AppError> {
    if self.named_label(project, &command.title).await?.is_some() {
      return Err(AppError::bad_request("label title already exists"));
    }
    let now = chrono::Utc::now().naive_utc();
    Ok(
      labels::ActiveModel {
        id: Set(gity_core::new_id()),
        project_id: Set(project),
        title: Set(command.title),
        color: Set(command.color),
        description: Set(command.description),
        created_at: Set(now),
        updated_at: Set(now),
      }
      .insert(&self.connection)
      .await?,
    )
  }
  pub async fn update_label(
    &self,
    project: i64,
    id: i64,
    command: LabelChanges,
  ) -> Result<labels::Model, AppError> {
    let row = self.label(project, id).await?;
    if let Change::Set(title) = &command.title
      && title != &row.title
      && self.named_label(project, title).await?.is_some()
    {
      return Err(AppError::bad_request("label title already exists"));
    }
    let mut row = row.into_active_model();
    if let Change::Set(v) = command.title {
      row.title = Set(v);
    }
    if let Change::Set(v) = command.color {
      row.color = Set(v);
    }
    if let Change::Set(v) = command.description {
      row.description = Set(v);
    }
    row.updated_at = Set(chrono::Utc::now().naive_utc());
    Ok(row.update(&self.connection).await?)
  }
  pub async fn delete_label(&self, project: i64, id: i64) -> Result<(), AppError> {
    let row = self.label(project, id).await?;
    let tx = self.connection.begin().await?;
    links::Entity::delete_many()
      .filter(links::Column::LabelId.eq(row.id))
      .exec(&tx)
      .await?;
    labels::Entity::delete_by_id(row.id).exec(&tx).await?;
    tx.commit().await?;
    Ok(())
  }
  pub async fn issue_label_rows(
    &self,
    project: i64,
    issues: Vec<i64>,
  ) -> Result<Vec<(i64, labels::Model)>, AppError> {
    if issues.is_empty() {
      return Ok(Vec::new());
    }
    let linked = links::Entity::find()
      .filter(links::Column::ProjectId.eq(project))
      .filter(links::Column::TargetType.eq("issue"))
      .filter(links::Column::TargetId.is_in(issues))
      .order_by_asc(links::Column::CreatedAt)
      .order_by_asc(links::Column::Id)
      .all(&self.connection)
      .await?;
    let models = labels::Entity::find()
      .filter(labels::Column::ProjectId.eq(project))
      .filter(labels::Column::Id.is_in(linked.iter().map(|l| l.label_id)))
      .all(&self.connection)
      .await?
      .into_iter()
      .map(|m| (m.id, m))
      .collect::<std::collections::HashMap<_, _>>();
    Ok(
      linked
        .into_iter()
        .filter_map(|l| models.get(&l.label_id).cloned().map(|m| (l.target_id, m)))
        .collect(),
    )
  }
  pub async fn replace_labels(
    &self,
    project: i64,
    iid: i64,
    wanted: Vec<(String, Option<String>)>,
  ) -> Result<(), AppError> {
    let issue = self.issue(project, iid).await?;
    let tx = self.connection.begin().await?;
    let mut attached = Vec::with_capacity(wanted.len());
    for (title, color) in wanted {
      attached.push(ensure(&tx, project, &title, color).await?.id);
    }
    let current = links::Entity::find()
      .filter(links::Column::ProjectId.eq(project))
      .filter(links::Column::TargetType.eq("issue"))
      .filter(links::Column::TargetId.eq(issue.id))
      .all(&tx)
      .await?;
    for link in &current {
      if !attached.contains(&link.label_id) {
        links::Entity::delete_by_id(link.id).exec(&tx).await?;
      }
    }
    let now = chrono::Utc::now().naive_utc();
    for label in attached {
      if current.iter().any(|link| link.label_id == label) {
        continue;
      }
      links::ActiveModel {
        id: Set(gity_core::new_id()),
        project_id: Set(project),
        label_id: Set(label),
        target_type: Set("issue".to_owned()),
        target_id: Set(issue.id),
        created_at: Set(now),
      }
      .insert(&tx)
      .await?;
    }
    tx.commit().await?;
    Ok(())
  }
}
async fn ensure<C: ConnectionTrait>(
  db: &C,
  project: i64,
  title: &str,
  color: Option<String>,
) -> Result<labels::Model, AppError> {
  if let Some(row) = labels::Entity::find()
    .filter(labels::Column::ProjectId.eq(project))
    .filter(labels::Column::Title.eq(title))
    .one(db)
    .await?
  {
    return Ok(row);
  }
  let now = chrono::Utc::now().naive_utc();
  Ok(
    labels::ActiveModel {
      id: Set(gity_core::new_id()),
      project_id: Set(project),
      title: Set(title.to_owned()),
      color: Set(color.unwrap_or_else(|| DEFAULT_COLOR.to_owned())),
      description: Set(None),
      created_at: Set(now),
      updated_at: Set(now),
    }
    .insert(db)
    .await?,
  )
}
