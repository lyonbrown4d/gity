//! Frozen latest schema: 认证会话.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum AuthSessions {
  Table,
  TokenHash,
  UserId,
  Kind,
  ExpiresAt,
  Family,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use AuthSessions::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("认证会话");
  table.col(
    ColumnDef::new(TokenHash)
      .string_len(64)
      .not_null()
      .comment("令牌哈希"),
  );
  table.col(
    columns::big_integer(UserId, backend)
      .not_null()
      .comment("用户标识"),
  );
  table.col(
    ColumnDef::new(Kind)
      .string_len(16)
      .not_null()
      .comment("类别"),
  );
  table.col(
    columns::timestamp(ExpiresAt, backend)
      .not_null()
      .comment("过期时间"),
  );
  table.col(
    ColumnDef::new(Family)
      .string_len(64)
      .not_null()
      .comment("会话令牌族"),
  );
  table.primary_key(Index::create().name("pk_auth_sessions").col(TokenHash));
  columns::finish_table(&mut table, backend, "utf8mb4_0900_ai_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  vec![]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
