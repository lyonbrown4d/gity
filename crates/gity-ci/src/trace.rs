use crate::AppendTrace;
use crate::repository::*;
use gity_core::{AppError, new_id};
use gity_entities::{project_job_logs as log, project_jobs as job};
use sea_orm::{Set, TransactionTrait, entity::prelude::*};
use serde_json::{Value, json};
impl CiRepository {
  pub async fn append_trace(&self, cmd: AppendTrace) -> Result<(), AppError> {
    if cmd.output.len() > 1024 * 1024 {
      return Err(AppError::bad_request("trace chunk exceeds 1 MiB"));
    }
    let tx = self.connection.begin().await?;
    let changed = job::Entity::update_many()
      .set(job::ActiveModel {
        updated_at: Set(time()),
        ..Default::default()
      })
      .filter(live(&cmd.lease))
      .exec(&tx)
      .await?
      .rows_affected;
    if changed != 1 {
      return Err(AppError::conflict("job lease changed"));
    }
    let row = job::Entity::find_by_id(cmd.lease.job_id)
      .one(&tx)
      .await?
      .ok_or_else(missing)?;
    let mut data: Value = serde_json::from_str(row.payload.as_deref().unwrap_or("{}"))
      .map_err(|_| AppError::internal("invalid persisted CI job payload"))?;
    let (output, pending) = redact_chunk(
      data["trace_pending"].as_str().unwrap_or_default(),
      &cmd.output,
      &cmd.masked_values,
      cmd.final_chunk,
    );
    data["trace_pending"] = json!(pending);
    job::Entity::update_many()
      .set(job::ActiveModel {
        payload: Set(Some(data.to_string())),
        ..Default::default()
      })
      .filter(job::Column::Id.eq(row.id))
      .exec(&tx)
      .await?;
    let old = log::Entity::find()
      .filter(log::Column::ProjectJobId.eq(row.id))
      .filter(log::Column::Attempt.eq(cmd.lease.attempt))
      .one(&tx)
      .await?;
    let mut combined = format!(
      "{}{}",
      old.as_ref().map(|l| l.output.as_str()).unwrap_or_default(),
      output
    );
    let truncated = combined.len() > 4 * 1024 * 1024
      || cmd.truncated
      || old.as_ref().is_some_and(|l| l.output_truncated != 0);
    if combined.len() > 4 * 1024 * 1024 {
      let mut end = 4 * 1024 * 1024;
      while !combined.is_char_boundary(end) {
        end -= 1;
      }
      combined.truncate(end);
    }
    if let Some(old) = old {
      log::Entity::update_many()
        .set(log::ActiveModel {
          output: Set(combined),
          output_truncated: Set(i32::from(truncated)),
          duration_millis: Set(cmd.duration_millis.max(0)),
          updated_at: Set(time()),
          ..Default::default()
        })
        .filter(log::Column::Id.eq(old.id))
        .exec(&tx)
        .await?;
    } else {
      log::ActiveModel {
        id: Set(new_id()),
        project_id: Set(row.project_id),
        project_job_id: Set(row.id),
        attempt: Set(cmd.lease.attempt),
        exit_code: Set(0),
        output: Set(combined),
        output_truncated: Set(i32::from(truncated)),
        duration_millis: Set(cmd.duration_millis.max(0)),
        created_at: Set(time()),
        updated_at: Set(time()),
      }
      .insert(&tx)
      .await?;
    }
    tx.commit().await?;
    Ok(())
  }
}
fn redact_chunk(
  pending: &str,
  input: &str,
  secrets: &[String],
  final_chunk: bool,
) -> (String, String) {
  let mut output = format!("{pending}{input}");
  let mut secrets = secrets.iter().filter(|s| !s.is_empty()).collect::<Vec<_>>();
  secrets.sort_by_key(|s| std::cmp::Reverse(s.len()));
  for s in &secrets {
    output = output.replace(s.as_str(), "[MASKED]");
  }
  let mut retained = 0;
  for secret in secrets {
    for (i, _) in secret.char_indices().skip(1) {
      if output.ends_with(&secret[..i]) {
        retained = retained.max(i);
      }
    }
  }
  if retained == 0 {
    return (output, String::new());
  }
  let pending = output.split_off(output.len() - retained);
  if final_chunk {
    output.push_str("[MASKED]");
    (output, String::new())
  } else {
    (output, pending)
  }
}
