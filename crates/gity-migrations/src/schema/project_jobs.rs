//! Frozen latest schema: 项目后台任务.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectJobs {
  Table,
  Id,
  ProjectId,
  Kind,
  Status,
  Payload,
  Result,
  Attempts,
  MaxAttempts,
  RunAfter,
  LockedBy,
  LockedUntil,
  LastError,
  CreatedAt,
  UpdatedAt,
  StartedAt,
  FinishedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectJobs::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("项目后台任务");
  table.col(
    columns::big_integer(Id, backend)
      .not_null()
      .comment("应用生成的正数雪花标识"),
  );
  table.col(
    columns::big_integer(ProjectId, backend)
      .not_null()
      .comment("项目标识"),
  );
  table.col(
    columns::string(Kind, backend, 64)
      .not_null()
      .comment("类别"),
  );
  table.col(
    columns::string(Status, backend, 64)
      .not_null()
      .comment("运行状态"),
  );
  table.col(columns::text(Payload, backend).comment("任务或事件载荷"));
  table.col(columns::text(Result, backend).comment("执行结果"));
  table.col(
    ColumnDef::new(Attempts)
      .integer()
      .not_null()
      .comment("尝试次数"),
  );
  table.col(
    ColumnDef::new(MaxAttempts)
      .integer()
      .not_null()
      .comment("最大尝试次数"),
  );
  table.col(
    columns::timestamp(RunAfter, backend)
      .not_null()
      .comment("计划执行时间"),
  );
  table.col(columns::string(LockedBy, backend, 255).comment("锁定执行者"));
  table.col(
    columns::timestamp(LockedUntil, backend)
      .not_null()
      .comment("锁定截止时间"),
  );
  table.col(columns::text(LastError, backend).comment("最近错误说明"));
  table.col(
    columns::timestamp(CreatedAt, backend)
      .not_null()
      .comment("创建时间"),
  );
  table.col(
    columns::timestamp(UpdatedAt, backend)
      .not_null()
      .comment("更新时间"),
  );
  table.col(
    columns::timestamp(StartedAt, backend)
      .not_null()
      .comment("开始时间"),
  );
  table.col(
    columns::timestamp(FinishedAt, backend)
      .not_null()
      .comment("结束时间"),
  );
  table.primary_key(Index::create().name("pk_project_jobs").col(Id));
  table.check(("ck_project_jobs_id_positive", Expr::col(Id).gt(0)));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_jobs_project_id")
      .from(Table, ProjectId)
      .to(
        super::projects::Projects::Table,
        super::projects::Projects::Id,
      )
      .on_delete(ForeignKeyAction::Cascade)
      .on_update(ForeignKeyAction::NoAction),
  );
  columns::finish_table(&mut table, backend, "utf8mb4_unicode_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use ProjectJobs::*;
  vec![
    (
      Index::create()
        .name("ix_project_jobs_locked")
        .table(Table)
        .col(LockedBy)
        .col(LockedUntil)
        .to_owned(),
      "按锁定执行者、锁定截止时间查询",
    ),
    (
      Index::create()
        .name("ix_project_jobs_queue")
        .table(Table)
        .col(Status)
        .col(RunAfter)
        .col(Kind)
        .to_owned(),
      "按运行状态、计划执行时间、类别查询",
    ),
  ]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
