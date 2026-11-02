//! Frozen latest schema: 任务制品.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectJobArtifacts {
  Table,
  Id,
  ProjectId,
  ProjectJobId,
  Name,
  FileName,
  FilePath,
  ContentType,
  ByteSize,
  StorageKey,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectJobArtifacts::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("任务制品");
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
    columns::string(Name, backend, 255)
      .not_null()
      .comment("名称"),
  );
  table.col(
    columns::string(FileName, backend, 255)
      .not_null()
      .comment("文件名称"),
  );
  table.col(
    columns::string(FilePath, backend, 512)
      .not_null()
      .comment("文件路径"),
  );
  table.col(
    columns::string(ContentType, backend, 255)
      .not_null()
      .comment("媒体类型"),
  );
  table.col(
    columns::big_integer(ByteSize, backend)
      .not_null()
      .comment("字节大小"),
  );
  table.col(
    columns::string(StorageKey, backend, 512)
      .not_null()
      .comment("对象存储键"),
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
  table.primary_key(Index::create().name("pk_project_job_artifacts").col(Id));
  table.check(("ck_project_job_artifacts_id_positive", Expr::col(Id).gt(0)));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_job_artifacts_project_id")
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
      .name("fk_project_job_artifacts_project_job_id")
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
  use ProjectJobArtifacts::*;
  vec![(
    Index::create()
      .name("ix_project_job_artifacts_job")
      .table(Table)
      .col(ProjectJobId)
      .to_owned(),
    "按后台任务标识查询",
  )]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
