use crate::{Placement, Replica};
use anyhow::{Context, Result, ensure};
use gity_entities::git_repository_placements as rows;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
  QuerySelect, Set,
  sea_query::{Expr, ExprTrait},
};

#[derive(Clone)]
pub(crate) struct Catalog {
  pub connection: DatabaseConnection,
}
impl Catalog {
  pub async fn get(&self, project: i64) -> Result<Option<Placement>> {
    rows::Entity::find_by_id(project)
      .one(&self.connection)
      .await?
      .map(decode)
      .transpose()
  }
  pub async fn create(&self, project: i64, ranked: Vec<String>, count: usize) -> Result<Placement> {
    ensure!(!ranked.is_empty(), "Git storage node list is empty");
    let replicas = ranked
      .iter()
      .skip(1)
      .take(count.saturating_sub(1))
      .cloned()
      .map(Replica::pending)
      .collect::<Vec<_>>();
    let row = rows::ActiveModel {
      project_id: Set(project),
      primary_node: Set(ranked[0].clone()),
      transition_target: Set(None),
      epoch: Set(1),
      generation: Set(0),
      pending_writes: Set(0),
      state: Set("ready".into()),
      replicas_json: Set(serde_json::to_string(&replicas)?),
      version: Set(0),
      last_error: Set(None),
      updated_at: Set(chrono::Utc::now().naive_utc()),
    };
    match row.insert(&self.connection).await {
      Ok(row) => decode(row),
      Err(error)
        if matches!(
          error.sql_err(),
          Some(sea_orm::SqlErr::UniqueConstraintViolation(_))
        ) =>
      {
        self
          .get(project)
          .await?
          .context("Git placement creation raced with removal")
      }
      Err(error) => Err(error.into()),
    }
  }
  pub async fn begin_write(&self, row: &Placement) -> Result<bool> {
    Ok(
      rows::Entity::update_many()
        .col_expr(
          rows::Column::PendingWrites,
          Expr::col(rows::Column::PendingWrites).add(1),
        )
        .col_expr(
          rows::Column::Version,
          Expr::col(rows::Column::Version).add(1),
        )
        .col_expr(
          rows::Column::UpdatedAt,
          Expr::value(chrono::Utc::now().naive_utc()),
        )
        .filter(rows::Column::ProjectId.eq(row.project_id))
        .filter(rows::Column::Epoch.eq(row.epoch))
        .filter(rows::Column::Version.eq(row.version))
        .filter(rows::Column::State.eq("ready"))
        .exec(&self.connection)
        .await?
        .rows_affected
        == 1,
    )
  }
  pub async fn finish_write(&self, project: i64, epoch: i64, node: &str) -> Result<()> {
    let changed = rows::Entity::update_many()
      .col_expr(
        rows::Column::PendingWrites,
        Expr::col(rows::Column::PendingWrites).sub(1),
      )
      .col_expr(
        rows::Column::Generation,
        Expr::col(rows::Column::Generation).add(1),
      )
      .col_expr(
        rows::Column::Version,
        Expr::col(rows::Column::Version).add(1),
      )
      .col_expr(
        rows::Column::UpdatedAt,
        Expr::value(chrono::Utc::now().naive_utc()),
      )
      .filter(rows::Column::ProjectId.eq(project))
      .filter(rows::Column::Epoch.eq(epoch))
      .filter(rows::Column::PrimaryNode.eq(node))
      .filter(rows::Column::PendingWrites.gt(0))
      .filter(rows::Column::State.eq("ready"))
      .exec(&self.connection)
      .await?
      .rows_affected;
    ensure!(
      changed == 1,
      "Git placement changed while confirming a write; controlled reconciliation is required"
    );
    Ok(())
  }
  pub async fn replace(&self, old: &Placement, new: &Placement) -> Result<bool> {
    Ok(
      rows::Entity::update_many()
        .set(rows::ActiveModel {
          primary_node: Set(new.primary_node.clone()),
          transition_target: Set(new.transition_target.clone()),
          epoch: Set(new.epoch),
          generation: Set(new.generation),
          pending_writes: Set(new.pending_writes),
          state: Set(new.state.clone()),
          replicas_json: Set(serde_json::to_string(&new.replicas)?),
          version: Set(old.version + 1),
          last_error: Set(new.last_error.clone()),
          updated_at: Set(chrono::Utc::now().naive_utc()),
          ..Default::default()
        })
        .filter(rows::Column::ProjectId.eq(old.project_id))
        .filter(rows::Column::Version.eq(old.version))
        .exec(&self.connection)
        .await?
        .rows_affected
        == 1,
    )
  }
  pub async fn scan(&self, limit: u64) -> Result<Vec<Placement>> {
    rows::Entity::find()
      .order_by_asc(rows::Column::UpdatedAt)
      .limit(limit)
      .all(&self.connection)
      .await?
      .into_iter()
      .map(decode)
      .collect()
  }
}
fn decode(row: rows::Model) -> Result<Placement> {
  Ok(Placement {
    project_id: row.project_id,
    primary_node: row.primary_node,
    transition_target: row.transition_target,
    epoch: row.epoch,
    generation: row.generation,
    pending_writes: row.pending_writes,
    state: row.state,
    replicas: serde_json::from_str(&row.replicas_json).context("invalid Git replica metadata")?,
    version: row.version,
    last_error: row.last_error,
  })
}
