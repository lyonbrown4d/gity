//! Durable, coalesced project search projection rebuild requests.
use gity_entities::search_index_jobs::{self as jobs, Column};
use sea_orm::{
  ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder,
  QuerySelect, Set,
  sea_query::{Condition, Expr, ExprTrait, OnConflict},
};

const LEASE_SECONDS: i64 = 300;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Claim {
  pub project_id: i64,
  pub token: String,
  pub generation: i64,
}

/// Record a dirty project in the caller's transaction. Repeated changes
/// coalesce into one row while advancing its generation atomically.
pub async fn mark_dirty<C: ConnectionTrait>(db: &C, project_id: i64) -> Result<(), DbErr> {
  let now = chrono::Utc::now().naive_utc();
  jobs::Entity::insert(jobs::ActiveModel {
    id: Set(gity_core::new_id()),
    project_id: Set(project_id),
    requested_generation: Set(1),
    completed_generation: Set(0),
    published_token: Set(None),
    published_generation: Set(0),
    lease_token: Set(None),
    lease_generation: Set(None),
    lease_expires_at: Set(None),
    updated_at: Set(now),
  })
  .on_conflict(
    OnConflict::column(Column::ProjectId)
      .value(
        Column::RequestedGeneration,
        Expr::col((jobs::Entity, Column::RequestedGeneration)).add(1),
      )
      .value(Column::UpdatedAt, now)
      .to_owned(),
  )
  .exec_without_returning(db)
  .await
  .map(|_| ())
}

pub struct Jobs {
  db: DatabaseConnection,
  namespace_prefix: String,
}

impl Jobs {
  pub fn new(db: DatabaseConnection) -> Self {
    Self {
      db,
      namespace_prefix: String::new(),
    }
  }

  pub fn with_namespace(db: DatabaseConnection, namespace_prefix: String) -> Self {
    Self {
      db,
      namespace_prefix,
    }
  }

  pub async fn mark_dirty(&self, project_id: i64) -> Result<(), DbErr> {
    mark_dirty(&self.db, project_id).await
  }

  /// Claim one eligible project using a conditional update as the lock. The
  /// bounded lease permits recovery after a worker exits without releasing it.
  pub async fn claim(&self) -> Result<Option<Claim>, DbErr> {
    let now = chrono::Utc::now().naive_utc();
    let candidates = jobs::Entity::find()
      .filter(
        Condition::all().add(
          Expr::col((jobs::Entity, Column::RequestedGeneration))
            .gt(Expr::col((jobs::Entity, Column::CompletedGeneration))),
        ),
      )
      .filter(
        Condition::any()
          .add(Column::LeaseToken.is_null())
          .add(Column::LeaseExpiresAt.lte(now)),
      )
      .order_by_asc(Column::UpdatedAt)
      .limit(32)
      .all(&self.db)
      .await?;

    for candidate in candidates {
      let token = format!("{}{}", self.namespace_prefix, uuid::Uuid::now_v7());
      let generation = candidate.requested_generation;
      let expiry = now + chrono::Duration::seconds(LEASE_SECONDS);
      let result = jobs::Entity::update_many()
        .col_expr(Column::LeaseToken, Expr::value(token.clone()))
        .col_expr(Column::LeaseGeneration, Expr::value(generation))
        .col_expr(Column::LeaseExpiresAt, Expr::value(expiry))
        .filter(Column::ProjectId.eq(candidate.project_id))
        .filter(Column::RequestedGeneration.eq(generation))
        .filter(
          Condition::all().add(
            Expr::col((jobs::Entity, Column::RequestedGeneration))
              .gt(Expr::col((jobs::Entity, Column::CompletedGeneration))),
          ),
        )
        .filter(
          Condition::any()
            .add(Column::LeaseToken.is_null())
            .add(Column::LeaseExpiresAt.lte(now)),
        )
        .exec(&self.db)
        .await?;
      if result.rows_affected == 1 {
        return Ok(Some(Claim {
          project_id: candidate.project_id,
          token,
          generation,
        }));
      }
    }
    Ok(None)
  }

  pub async fn complete(&self, claim: &Claim) -> Result<bool, DbErr> {
    let now = chrono::Utc::now().naive_utc();
    let result = jobs::Entity::update_many()
      .col_expr(Column::CompletedGeneration, Expr::value(claim.generation))
      .col_expr(Column::PublishedToken, Expr::value(claim.token.clone()))
      .col_expr(Column::PublishedGeneration, Expr::value(claim.generation))
      .col_expr(Column::LeaseToken, Expr::value(None::<String>))
      .col_expr(Column::LeaseGeneration, Expr::value(None::<i64>))
      .col_expr(
        Column::LeaseExpiresAt,
        Expr::value(None::<chrono::NaiveDateTime>),
      )
      .filter(Column::ProjectId.eq(claim.project_id))
      .filter(Column::LeaseToken.eq(claim.token.clone()))
      .filter(Column::LeaseGeneration.eq(claim.generation))
      .filter(Column::LeaseExpiresAt.gt(now))
      .exec(&self.db)
      .await?;
    Ok(result.rows_affected == 1)
  }

  pub async fn retry(&self, claim: &Claim) -> Result<(), DbErr> {
    jobs::Entity::update_many()
      .col_expr(
        Column::LeaseToken,
        Expr::value(uuid::Uuid::now_v7().to_string()),
      )
      .col_expr(Column::LeaseGeneration, Expr::value(None::<i64>))
      .col_expr(
        Column::LeaseExpiresAt,
        Expr::value(chrono::Utc::now().naive_utc() + chrono::Duration::seconds(30)),
      )
      .col_expr(
        Column::UpdatedAt,
        Expr::value(chrono::Utc::now().naive_utc()),
      )
      .filter(Column::ProjectId.eq(claim.project_id))
      .filter(Column::LeaseToken.eq(claim.token.clone()))
      .filter(Column::LeaseGeneration.eq(claim.generation))
      .exec(&self.db)
      .await?;
    Ok(())
  }

  /// Tell whether this lease may still write to its token-isolated namespace.
  pub async fn is_current(&self, claim: &Claim) -> Result<bool, DbErr> {
    let now = chrono::Utc::now().naive_utc();
    Ok(
      jobs::Entity::find()
        .filter(Column::ProjectId.eq(claim.project_id))
        .filter(Column::LeaseToken.eq(claim.token.clone()))
        .filter(Column::LeaseGeneration.eq(claim.generation))
        .filter(Column::LeaseExpiresAt.gt(now))
        .one(&self.db)
        .await?
        .is_some(),
    )
  }

  /// Report projection lag without exposing resource counts to callers.
  pub async fn has_pending(&self, projects: &[i64]) -> Result<bool, DbErr> {
    for batch in projects.chunks(300) {
      if jobs::Entity::find()
        .filter(Column::ProjectId.is_in(batch.iter().copied()))
        .filter(
          Expr::col((jobs::Entity, Column::RequestedGeneration))
            .gt(Expr::col((jobs::Entity, Column::CompletedGeneration))),
        )
        .one(&self.db)
        .await?
        .is_some()
      {
        return Ok(true);
      }
    }
    Ok(false)
  }

  /// Return the last successfully published namespace token for each project.
  pub async fn published(
    &self,
    project_ids: &[i64],
  ) -> Result<std::collections::HashMap<i64, String>, DbErr> {
    let unique_ids = project_ids
      .iter()
      .copied()
      .collect::<std::collections::HashSet<_>>();
    let ids = unique_ids.into_iter().collect::<Vec<_>>();
    let mut published = std::collections::HashMap::new();
    for batch in ids.chunks(300) {
      for row in jobs::Entity::find()
        .filter(Column::ProjectId.is_in(batch.iter().copied()))
        .filter(Column::PublishedToken.is_not_null())
        .all(&self.db)
        .await?
      {
        if let Some(token) = row.published_token {
          published.insert(row.project_id, token);
        }
      }
    }
    Ok(published)
  }
}
