use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum MergeOperations {
  Table,
  Id,
  OperationKey,
  ProjectId,
  MergeRequestId,
  ActorUserId,
  SourceBranch,
  TargetBranch,
  SourceSha,
  TargetSha,
  Message,
  AuthorName,
  AuthorEmail,
  Status,
  ResultSha,
  Rejection,
  RejectionActorUserId,
  EventsDelivered,
  CreatedAt,
  UpdatedAt,
}
#[derive(DeriveIden)]
enum ProjectMergeRequests {
  Table,
  ActiveMergeOperationId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    use MergeOperations::*;
    let mut table = sea_query::Table::create();
    table
      .table(Table)
      .comment("Durable merge intents and completion outbox");
    table.col(
      ColumnDef::new(Id)
        .big_integer()
        .not_null()
        .primary_key()
        .comment("Application snowflake ID"),
    );
    for (column, description) in [
      (ProjectId, "Owning project ID"),
      (MergeRequestId, "Merge request ID"),
      (ActorUserId, "Authorizing actor ID"),
    ] {
      table.col(
        ColumnDef::new(column)
          .big_integer()
          .not_null()
          .comment(description),
      );
    }
    for (column, description) in [
      (OperationKey, "Immutable UUID v7 operation identity"),
      (SourceBranch, "Pinned source branch"),
      (TargetBranch, "Pinned target branch"),
      (SourceSha, "Expected source commit"),
      (TargetSha, "Expected target commit"),
      (AuthorName, "Commit author name"),
      (AuthorEmail, "Commit author email"),
      (Status, "pending, applied, or rejected"),
    ] {
      table.col(
        ColumnDef::new(column)
          .string()
          .not_null()
          .comment(description),
      );
    }
    table.col(
      ColumnDef::new(Message)
        .text()
        .not_null()
        .comment("Pinned merge commit message"),
    );
    table.col(
      ColumnDef::new(ResultSha)
        .string()
        .null()
        .comment("Exact recorded merge commit"),
    );
    table.col(
      ColumnDef::new(Rejection)
        .text()
        .null()
        .comment("Durable Git rejection reason"),
    );
    table.col(
      ColumnDef::new(RejectionActorUserId)
        .big_integer()
        .null()
        .comment("Actor recording the rejected or aborted outcome"),
    );
    table.col(
      ColumnDef::new(EventsDelivered)
        .boolean()
        .not_null()
        .default(false)
        .comment("Idempotent CI outbox delivery completed"),
    );
    for column in [CreatedAt, UpdatedAt] {
      table.col(
        ColumnDef::new(column)
          .date_time()
          .not_null()
          .comment("UTC timestamp"),
      );
    }
    manager.create_table(table.clone()).await?;
    manager
      .create_index(
        Index::create()
          .name("uq_merge_operation_key")
          .table(Table)
          .col(OperationKey)
          .unique()
          .to_owned(),
      )
      .await?;
    manager
      .create_index(
        Index::create()
          .name("idx_merge_operation_recovery")
          .table(Table)
          .col(Status)
          .col(EventsDelivered)
          .col(Id)
          .to_owned(),
      )
      .await?;
    manager
      .alter_table(
        sea_query::Table::alter()
          .table(ProjectMergeRequests::Table)
          .add_column(
            ColumnDef::new(ProjectMergeRequests::ActiveMergeOperationId)
              .big_integer()
              .null(),
          )
          .to_owned(),
      )
      .await?;
    if manager.get_database_backend() == sea_orm_migration::sea_orm::DbBackend::Postgres {
      // SeaQuery has no COMMENT ON builder; these fixed DDL identifiers and
      // escaped schema descriptions are the only backend-specific SQL here.
      let connection = manager.get_connection();
      connection
        .execute_unprepared(
          "COMMENT ON TABLE merge_operations IS 'Durable merge intents and completion outbox'",
        )
        .await?;
      for column in table.get_columns() {
        let description = column
          .get_column_spec()
          .comment
          .as_ref()
          .unwrap()
          .replace('\'', "''");
        connection
          .execute_unprepared(&format!(
            "COMMENT ON COLUMN merge_operations.\"{}\" IS '{}'",
            column.get_column_name(),
            description
          ))
          .await?;
      }
      connection.execute_unprepared("COMMENT ON COLUMN project_merge_requests.active_merge_operation_id IS 'Atomically claimed durable merge operation ID'").await?;
      connection
        .execute_unprepared(
          "COMMENT ON INDEX uq_merge_operation_key IS 'One immutable operation per UUID'",
        )
        .await?;
      connection.execute_unprepared("COMMENT ON INDEX idx_merge_operation_recovery IS 'Pending reconciliation and outbox scan'").await?;
    }
    Ok(())
  }
  async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
    Err(DbErr::Migration(
      "merge operation receipts cannot be discarded; restore a backup".into(),
    ))
  }
}
