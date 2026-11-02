//! Frozen latest schema: 用户.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum Users {
  Table,
  Id,
  Username,
  DisplayName,
  Email,
  CreatedAt,
  UpdatedAt,
  IsSuperAdmin,
  Status,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use Users::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("用户");
  table.col(
    columns::big_integer(Id, backend)
      .not_null()
      .comment("应用生成的正数雪花标识"),
  );
  table.col(
    columns::string(Username, backend, 255)
      .not_null()
      .comment("用户名"),
  );
  table.col(
    columns::string(DisplayName, backend, 255)
      .not_null()
      .comment("显示名称"),
  );
  table.col(columns::string(Email, backend, 320).comment("电子邮箱"));
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
    ColumnDef::new(IsSuperAdmin)
      .integer()
      .not_null()
      .default(0i64)
      .comment("是否为超级管理员"),
  );
  table.col(
    columns::string(Status, backend, 32)
      .not_null()
      .default("active")
      .comment("运行状态"),
  );
  table.primary_key(Index::create().name("pk_users").col(Id));
  table.check(("ck_users_id_positive", Expr::col(Id).gt(0)));
  columns::finish_table(&mut table, backend, "utf8mb4_unicode_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use Users::*;
  vec![
    (
      Index::create()
        .name("ux_users_1")
        .table(Table)
        .unique()
        .col(Username)
        .to_owned(),
      "保证用户名唯一",
    ),
    (
      Index::create()
        .name("ux_users_2")
        .table(Table)
        .unique()
        .col(Email)
        .to_owned(),
      "保证电子邮箱唯一",
    ),
  ]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
