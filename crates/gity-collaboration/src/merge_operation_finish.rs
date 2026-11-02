use crate::CollaborationService;
use gity_core::AppError;
use gity_entities::{merge_operations as operations, project_merge_requests as requests};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
  sea_query::Expr,
};

pub struct MergeRejection {
  pub reason: String,
  pub actor_user_id: i64,
}

impl CollaborationService {
  pub async fn pending_merge_operations(
    &self,
    after: i64,
    limit: u64,
  ) -> Result<Vec<operations::Model>, AppError> {
    Ok(
      operations::Entity::find()
        .filter(operations::Column::Id.gt(after))
        .filter(
          sea_orm::Condition::any()
            .add(operations::Column::Status.eq("pending"))
            .add(
              sea_orm::Condition::all()
                .add(operations::Column::Status.eq("applied"))
                .add(operations::Column::EventsDelivered.eq(false)),
            ),
        )
        .order_by_asc(operations::Column::Id)
        .limit(limit)
        .all(&self.repository.connection)
        .await?,
    )
  }

  /// Persist only a proven Git outcome. MR state, journal, audit and notification
  /// become visible together; repeated completion never duplicates side effects.
  pub async fn finish_merge_operation(
    &self,
    id: i64,
    outcome: Result<String, MergeRejection>,
  ) -> Result<operations::Model, AppError> {
    self.lock_merge_operation(id).await?.finish(outcome).await
  }

  pub async fn merge_events_delivered(&self, id: i64) -> Result<(), AppError> {
    operations::Entity::update_many()
      .col_expr(operations::Column::EventsDelivered, Expr::value(true))
      .filter(operations::Column::Id.eq(id))
      .filter(operations::Column::Status.eq("applied"))
      .exec(&self.repository.connection)
      .await?;
    Ok(())
  }
}

impl crate::MergeOperationGuard {
  pub async fn finish(
    self,
    outcome: Result<String, MergeRejection>,
  ) -> Result<operations::Model, AppError> {
    let Self { tx, operation: op } = self;
    let id = op.id;
    if op.status != "pending" {
      let same = match &outcome {
        Ok(sha) => op.status == "applied" && op.result_sha.as_ref() == Some(sha),
        Err(reason) => {
          op.status == "rejected"
            && op.rejection.as_ref() == Some(&reason.reason)
            && op.rejection_actor_user_id == Some(reason.actor_user_id)
        }
      };
      if !same {
        return Err(AppError::conflict("contradictory merge operation outcome"));
      }
      tx.commit().await?;
      return Ok(op);
    }
    let mr = requests::Entity::find_by_id(op.merge_request_id)
      .one(&tx)
      .await?
      .ok_or_else(|| AppError::not_found("merge request not found"))?;
    if mr.active_merge_operation_id != Some(id) {
      return Err(AppError::conflict(
        "merge request operation binding changed",
      ));
    }
    let now = chrono::Utc::now().naive_utc();
    let mut change = requests::ActiveModel {
      id: Set(mr.id),
      active_merge_operation_id: Set(None),
      updated_at: Set(now),
      ..Default::default()
    };
    let (status, result_sha, rejection, rejection_actor) = match outcome {
      Ok(sha) => {
        change.state = Set("merged".into());
        change.merged_at = Set(Some(now));
        change.merged_by_user_id = Set(Some(op.actor_user_id));
        change.merge_commit_sha = Set(Some(sha.clone()));
        crate::merge_operation_events::record(&tx, &op, &mr, &sha).await?;
        ("applied", Some(sha), None, None)
      }
      Err(reason) => {
        crate::merge_operation_events::rejected(&tx, &op, &reason.reason, reason.actor_user_id)
          .await?;
        (
          "rejected",
          None,
          Some(reason.reason),
          Some(reason.actor_user_id),
        )
      }
    };
    change.update(&tx).await?;
    let result = operations::ActiveModel {
      id: Set(id),
      status: Set(status.into()),
      result_sha: Set(result_sha),
      rejection: Set(rejection),
      rejection_actor_user_id: Set(rejection_actor),
      updated_at: Set(now),
      ..Default::default()
    }
    .update(&tx)
    .await?;
    tx.commit().await?;
    Ok(result)
  }
}
