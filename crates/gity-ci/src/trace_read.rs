use crate::{CiService, repository::CiRepository};
use gity_core::AppError;
use gity_entities::project_job_logs as log;
use sea_orm::{DbBackend, FromQueryResult, QueryOrder, QuerySelect, entity::prelude::*};

pub struct TracePage {
  pub attempt: i32,
  pub log: Option<log::Model>,
  /// Metadata only: output is always empty, never read from persistence.
  pub attempt_summaries: Vec<log::Model>,
  pub offset: u64,
  pub next_offset: u64,
  pub total_bytes: u64,
  pub has_more: bool,
  pub truncated: bool,
}

#[derive(FromQueryResult)]
struct Bytes {
  output_bytes: Vec<u8>,
  total_bytes: i64,
}

impl CiService {
  /// Read at most 256 KiB from one attempt, without loading other trace bodies.
  pub async fn trace_page(
    &self,
    project: i64,
    job: i64,
    attempt: Option<i32>,
    offset: u64,
    limit: u64,
  ) -> Result<TracePage, AppError> {
    if attempt.is_some_and(|value| value < 1)
      || !(4..=256 * 1024).contains(&limit)
      || offset > i64::MAX as u64 - 1
    {
      return Err(AppError::bad_request(
        "invalid trace attempt, offset or limit (4–262144 bytes)",
      ));
    }
    self
      .repository
      .trace_page(project, job, attempt, offset, limit)
      .await
  }
}

impl CiRepository {
  async fn trace_page(
    &self,
    project: i64,
    job: i64,
    attempt: Option<i32>,
    offset: u64,
    limit: u64,
  ) -> Result<TracePage, AppError> {
    let query = log::Entity::find()
      .filter(log::Column::ProjectId.eq(project))
      .filter(log::Column::ProjectJobId.eq(job));
    let summaries = query
      .clone()
      .select_only()
      .columns([
        log::Column::Id,
        log::Column::ProjectId,
        log::Column::ProjectJobId,
        log::Column::Attempt,
        log::Column::ExitCode,
        log::Column::OutputTruncated,
        log::Column::DurationMillis,
        log::Column::CreatedAt,
        log::Column::UpdatedAt,
      ])
      .column_as(Expr::value(""), log::Column::Output)
      .order_by_asc(log::Column::Attempt)
      .order_by_asc(log::Column::Id)
      .all(&self.connection)
      .await?;
    let selected = attempt.unwrap_or_else(|| summaries.last().map_or(1, |row| row.attempt));
    let mut log = summaries
      .iter()
      .rev()
      .find(|row| row.attempt == selected)
      .cloned();
    let (mut actual_offset, mut total, mut output) = (offset, 0, String::new());
    if let Some(row) = &log {
      // Byte offsets must mean the same thing on every supported database.
      // SQL text substring counts characters; these fixed dialect expressions
      // convert to bytes before slicing, keeping the returned allocation bounded.
      let (slice, length) = match self.connection.get_database_backend() {
        DbBackend::Sqlite => ("substr(CAST(? AS BLOB), ?, ?)", "length(CAST(? AS BLOB))"),
        DbBackend::Postgres => (
          "substring(convert_to($1, 'UTF8') from $2 for $3)",
          "CAST(octet_length($1) AS BIGINT)",
        ),
        DbBackend::MySql => (
          "substring(CAST(? AS BINARY), ?, ?)",
          "CAST(octet_length(?) AS SIGNED)",
        ),
        _ => return Err(AppError::internal("unsupported trace database")),
      };
      let bytes = query
        .filter(log::Column::Id.eq(row.id))
        .select_only()
        .column_as(
          Expr::cust_with_exprs(
            slice,
            [
              Expr::col(log::Column::Output),
              // PostgreSQL's bytea substring takes i32 positions. Persisted traces
              // are capped at 4 MiB, so larger positions are equivalent empty reads.
              Expr::val((offset + 1).min(i32::MAX as u64) as i32),
              Expr::val((limit + 6) as i32),
            ],
          ),
          "output_bytes",
        )
        .column_as(
          Expr::cust_with_expr(length, Expr::col(log::Column::Output)),
          "total_bytes",
        )
        .into_model::<Bytes>()
        .one(&self.connection)
        .await?;
      if let Some(bytes) = bytes {
        total = bytes.total_bytes.max(0) as u64;
        let (skip, text) = utf8_page(&bytes.output_bytes, limit as usize);
        actual_offset = offset.saturating_add(skip as u64).min(total);
        output = text;
      }
    }
    actual_offset = actual_offset.min(total);
    let next_offset = actual_offset + output.len() as u64;
    let truncated = log.as_ref().is_some_and(|row| row.output_truncated != 0);
    if let Some(row) = &mut log {
      row.output = output;
    }
    Ok(TracePage {
      attempt: selected,
      log,
      attempt_summaries: summaries,
      offset: actual_offset,
      next_offset,
      total_bytes: total,
      has_more: next_offset < total,
      truncated,
    })
  }
}

fn utf8_page(bytes: &[u8], limit: usize) -> (usize, String) {
  // An arbitrary caller offset can begin inside a UTF-8 character. Advance to
  // its next boundary; next_offset always points to a complete character.
  let skip = bytes
    .iter()
    .take_while(|byte| (**byte & 0xc0) == 0x80)
    .count();
  let body = &bytes[skip..bytes.len().min(skip + limit)];
  let end = std::str::from_utf8(body)
    .err()
    .map_or(body.len(), |error| error.valid_up_to());
  (skip, String::from_utf8_lossy(&body[..end]).into_owned())
}
