//! Frozen latest schema: GitLFS对象.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectLfsObjects {
  Table,
  Id,
  ProjectId,
  Oid,
  ByteSize,
  StorageKey,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectLfsObjects::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("GitLFS对象");
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
    columns::string(Oid, backend, 255)
      .not_null()
      .comment("LFS对象哈希"),
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
  table.primary_key(Index::create().name("pk_project_lfs_objects").col(Id));
  table.check(("ck_project_lfs_objects_id_positive", Expr::col(Id).gt(0)));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_lfs_objects_project_id")
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
  use ProjectLfsObjects::*;
  vec![
    (
      Index::create()
        .name("ux_project_lfs_objects_1")
        .table(Table)
        .unique()
        .col(StorageKey)
        .to_owned(),
      "保证对象存储键唯一",
    ),
    (
      Index::create()
        .name("ux_project_lfs_objects_project_oid")
        .table(Table)
        .unique()
        .col(ProjectId)
        .col(Oid)
        .to_owned(),
      "保证项目标识、LFS对象哈希唯一",
    ),
  ]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
