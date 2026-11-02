//! Frozen latest schema: 软件包文件.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectPackageFiles {
  Table,
  Id,
  ProjectPackageVersionId,
  FileName,
  FilePath,
  ContentType,
  ByteSize,
  StorageKey,
  CreatedAt,
  UpdatedAt,
  Sha256,
  CreatedByUserId,
  Source,
  PipelineId,
  JobId,
  RegistryMetadata,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectPackageFiles::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("软件包文件");
  table.col(
    columns::big_integer(Id, backend)
      .not_null()
      .comment("应用生成的正数雪花标识"),
  );
  table.col(
    columns::big_integer(ProjectPackageVersionId, backend)
      .not_null()
      .comment("软件包版本标识"),
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
  table.col(columns::string(ContentType, backend, 255).comment("媒体类型"));
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
  table.col(
    columns::string(Sha256, backend, 64)
      .not_null()
      .default("")
      .comment("SHA256校验值"),
  );
  table.col(columns::big_integer(CreatedByUserId, backend).comment("创建者用户标识"));
  table.col(
    columns::string(Source, backend, 32)
      .not_null()
      .default("manual")
      .comment("来源"),
  );
  table.col(columns::big_integer(PipelineId, backend).comment("流水线标识"));
  table.col(columns::big_integer(JobId, backend).comment("任务标识"));
  table.col(columns::text(RegistryMetadata, backend).comment("包协议元数据"));
  table.primary_key(Index::create().name("pk_project_package_files").col(Id));
  table.check(("ck_project_package_files_id_positive", Expr::col(Id).gt(0)));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_package_files_project_package_version_id")
      .from(Table, ProjectPackageVersionId)
      .to(
        super::project_package_versions::ProjectPackageVersions::Table,
        super::project_package_versions::ProjectPackageVersions::Id,
      )
      .on_delete(ForeignKeyAction::Cascade)
      .on_update(ForeignKeyAction::NoAction),
  );
  columns::finish_table(&mut table, backend, "utf8mb4_unicode_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use ProjectPackageFiles::*;
  vec![
    (
      Index::create()
        .name("ux_project_package_files_1")
        .table(Table)
        .unique()
        .col(StorageKey)
        .to_owned(),
      "保证对象存储键唯一",
    ),
    (
      Index::create()
        .name("ux_project_package_files_version_path")
        .table(Table)
        .unique()
        .col(ProjectPackageVersionId)
        .col(FilePath)
        .to_owned(),
      "保证软件包版本标识、文件路径唯一",
    ),
    (
      Index::create()
        .name("ix_project_package_files_sha256")
        .table(Table)
        .col(Sha256)
        .to_owned(),
      "按SHA256校验值查询",
    ),
    (
      Index::create()
        .name("ix_project_package_files_created_by")
        .table(Table)
        .col(CreatedByUserId)
        .to_owned(),
      "按创建者用户标识查询",
    ),
  ]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
