//! Frozen latest schema: 组织.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum Organizations {
  Table,
  Id,
  Name,
  PathKey,
  FullPath,
  Description,
  Visibility,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use Organizations::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("组织");
  table.col(
    columns::big_integer(Id, backend)
      .not_null()
      .comment("应用生成的正数雪花标识"),
  );
  table.col(
    columns::string(Name, backend, 255)
      .not_null()
      .comment("名称"),
  );
  table.col(
    columns::string(PathKey, backend, 255)
      .not_null()
      .comment("路径键"),
  );
  table.col(
    columns::string(FullPath, backend, 512)
      .not_null()
      .comment("完整路径"),
  );
  table.col(columns::text(Description, backend).comment("说明"));
  table.col(
    columns::string(Visibility, backend, 64)
      .not_null()
      .comment("可见范围"),
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
  table.primary_key(Index::create().name("pk_organizations").col(Id));
  table.check(("ck_organizations_id_positive", Expr::col(Id).gt(0)));
  columns::finish_table(&mut table, backend, "utf8mb4_unicode_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use Organizations::*;
  vec![
    (
      Index::create()
        .name("ux_organizations_1")
        .table(Table)
        .unique()
        .col(PathKey)
        .to_owned(),
      "保证路径键唯一",
    ),
    (
      Index::create()
        .name("ux_organizations_2")
        .table(Table)
        .unique()
        .col(FullPath)
        .to_owned(),
      "保证完整路径唯一",
    ),
  ]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
