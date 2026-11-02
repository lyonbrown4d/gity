//! Frozen latest schema: 项目流水线.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectPipelines {
  Table,
  Id,
  ProjectId,
  Iid,
  Name,
  Source,
  RefName,
  CommitSha,
  Status,
  ConfigSource,
  ConfigContent,
  CreatedAt,
  UpdatedAt,
  StartedAt,
  FinishedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectPipelines::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("项目流水线");
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
    columns::big_integer(Iid, backend)
      .not_null()
      .comment("项目内编号"),
  );
  table.col(
    columns::string(Name, backend, 255)
      .not_null()
      .comment("名称"),
  );
  table.col(
    columns::string(Source, backend, 64)
      .not_null()
      .comment("来源"),
  );
  table.col(columns::string(RefName, backend, 255).comment("Git引用名称"));
  table.col(columns::string(CommitSha, backend, 255).comment("提交哈希"));
  table.col(
    columns::string(Status, backend, 64)
      .not_null()
      .comment("运行状态"),
  );
  table.col(
    columns::string(ConfigSource, backend, 64)
      .not_null()
      .comment("配置来源"),
  );
  table.col(
    columns::text(ConfigContent, backend)
      .not_null()
      .comment("流水线配置正文"),
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
  table.primary_key(Index::create().name("pk_project_pipelines").col(Id));
  table.check(("ck_project_pipelines_id_positive", Expr::col(Id).gt(0)));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_pipelines_project_id")
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
  use ProjectPipelines::*;
  vec![
    (
      Index::create()
        .name("ix_project_pipelines_project_ref")
        .table(Table)
        .col(ProjectId)
        .col(RefName)
        .to_owned(),
      "按项目标识、Git引用名称查询",
    ),
    (
      Index::create()
        .name("ix_project_pipelines_project_status")
        .table(Table)
        .col(ProjectId)
        .col(Status)
        .to_owned(),
      "按项目标识、运行状态查询",
    ),
    (
      Index::create()
        .name("ux_project_pipelines_project_iid")
        .table(Table)
        .unique()
        .col(ProjectId)
        .col(Iid)
        .to_owned(),
      "保证项目标识、项目内编号唯一",
    ),
  ]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
