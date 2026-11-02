//! Frozen latest schema: 任务执行日志.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectJobLogs {
  Table,
  Id,
  ProjectId,
  ProjectJobId,
  Attempt,
  ExitCode,
  Output,
  OutputTruncated,
  DurationMillis,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectJobLogs::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("任务执行日志");
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
    columns::big_integer(ProjectJobId, backend)
      .not_null()
      .comment("后台任务标识"),
  );
  table.col(
    ColumnDef::new(Attempt)
      .integer()
      .not_null()
      .comment("执行次数"),
  );
  table.col(
    ColumnDef::new(ExitCode)
      .integer()
      .not_null()
      .comment("进程退出码"),
  );
  table.col(
    columns::text(Output, backend)
      .not_null()
      .comment("执行输出"),
  );
  table.col(
    ColumnDef::new(OutputTruncated)
      .integer()
      .not_null()
      .comment("输出是否已截断"),
  );
  table.col(
    columns::big_integer(DurationMillis, backend)
      .not_null()
      .comment("执行时长毫秒"),
  );
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
  table.primary_key(Index::create().name("pk_project_job_logs").col(Id));
  table.check(("ck_project_job_logs_id_positive", Expr::col(Id).gt(0)));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_job_logs_project_id")
      .from(Table, ProjectId)
      .to(
        super::projects::Projects::Table,
        super::projects::Projects::Id,
      )
      .on_delete(ForeignKeyAction::Cascade)
      .on_update(ForeignKeyAction::NoAction),
  );
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_job_logs_project_job_id")
      .from(Table, ProjectJobId)
      .to(
        super::project_jobs::ProjectJobs::Table,
        super::project_jobs::ProjectJobs::Id,
      )
      .on_delete(ForeignKeyAction::Cascade)
      .on_update(ForeignKeyAction::NoAction),
  );
  columns::finish_table(&mut table, backend, "utf8mb4_unicode_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use ProjectJobLogs::*;
  vec![(
    Index::create()
      .name("ux_project_job_logs_job_attempt")
      .table(Table)
      .unique()
      .col(ProjectJobId)
      .col(Attempt)
      .to_owned(),
    "保证后台任务标识、执行次数唯一",
  )]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
