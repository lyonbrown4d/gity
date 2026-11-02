//! Frozen latest schema: 制品对象清理队列.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ArtifactStorageDeletions {
  Table,
  Id,
  StorageKey,
  Attempts,
  CreatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ArtifactStorageDeletions::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("制品对象清理队列");
  table.col(
    columns::big_integer(Id, backend)
      .not_null()
      .comment("应用生成的正数雪花标识"),
  );
  table.col(
    columns::string(StorageKey, backend, 512)
      .not_null()
      .comment("对象存储键"),
  );
  table.col(
    columns::big_integer(Attempts, backend)
      .not_null()
      .default(0i64)
      .comment("尝试次数"),
  );
  table.col(
    columns::timestamp(CreatedAt, backend)
      .not_null()
      .comment("创建时间"),
  );
  table.primary_key(
    Index::create()
      .name("pk_artifact_storage_deletions")
      .col(Id),
  );
  table.check((
    "ck_artifact_storage_deletions_id_positive",
    Expr::col(Id).gt(0),
  ));
  columns::finish_table(&mut table, backend, "utf8mb4_unicode_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use ArtifactStorageDeletions::*;
  vec![(
    Index::create()
      .name("ux_artifact_storage_deletions_1")
      .table(Table)
      .unique()
      .col(StorageKey)
      .to_owned(),
    "保证对象存储键唯一",
  )]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
