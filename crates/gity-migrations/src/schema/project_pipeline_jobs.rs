//! Frozen latest schema: 流水线任务配置.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectPipelineJobs {
  Table,
  Id,
  ProjectId,
  PipelineId,
  ProjectJobId,
  Name,
  Stage,
  Needs,
  Image,
  Script,
  Artifacts,
  SortOrder,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectPipelineJobs::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("流水线任务配置");
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
    columns::big_integer(PipelineId, backend)
      .not_null()
      .comment("流水线标识"),
  );
  table.col(
    columns::big_integer(ProjectJobId, backend)
      .not_null()
      .comment("后台任务标识"),
  );
  table.col(
    columns::string(Name, backend, 255)
      .not_null()
      .comment("名称"),
  );
  table.col(
    columns::string(Stage, backend, 255)
      .not_null()
      .comment("流水线阶段"),
  );
  table.col(columns::text(Needs, backend).comment("前置任务配置"));
  table.col(columns::string(Image, backend, 255).comment("执行镜像"));
  table.col(
    columns::text(Script, backend)
      .not_null()
      .comment("执行脚本"),
  );
  table.col(columns::text(Artifacts, backend).comment("制品配置"));
  table.col(
    ColumnDef::new(SortOrder)
      .integer()
      .not_null()
      .comment("排序序号"),
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
  table.primary_key(Index::create().name("pk_project_pipeline_jobs").col(Id));
  table.check(("ck_project_pipeline_jobs_id_positive", Expr::col(Id).gt(0)));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_pipeline_jobs_project_id")
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
      .name("fk_project_pipeline_jobs_pipeline_id")
      .from(Table, PipelineId)
      .to(
        super::project_pipelines::ProjectPipelines::Table,
        super::project_pipelines::ProjectPipelines::Id,
      )
      .on_delete(ForeignKeyAction::Cascade)
      .on_update(ForeignKeyAction::NoAction),
  );
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_pipeline_jobs_project_job_id")
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
  use ProjectPipelineJobs::*;
  vec![(
    Index::create()
      .name("ix_project_pipeline_jobs_pipeline_stage_order")
      .table(Table)
      .col(PipelineId)
      .col(Stage)
      .col(SortOrder)
      .to_owned(),
    "按流水线标识、流水线阶段、排序序号查询",
  )]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
