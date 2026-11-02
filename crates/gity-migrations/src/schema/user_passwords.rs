//! Frozen latest schema: 用户密码凭据.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum UserPasswords {
  Table,
  UserId,
  PasswordHash,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use UserPasswords::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("用户密码凭据");
  table.col(
    columns::big_integer(UserId, backend)
      .not_null()
      .comment("用户标识"),
  );
  table.col(
    columns::text(PasswordHash, backend)
      .not_null()
      .comment("密码哈希"),
  );
  table.primary_key(Index::create().name("pk_user_passwords").col(UserId));
  table.check((
    "ck_user_passwords_user_id_positive",
    Expr::col(UserId).gt(0),
  ));
  columns::finish_table(&mut table, backend, "utf8mb4_0900_ai_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  vec![]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
