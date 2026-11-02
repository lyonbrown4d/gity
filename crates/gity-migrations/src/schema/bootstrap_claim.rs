//! Frozen latest schema: 初始化占用凭据.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum BootstrapClaim {
  Table,
  Id,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use BootstrapClaim::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("初始化占用凭据");
  table.col(
    columns::big_integer(Id, backend)
      .not_null()
      .comment("初始化占用键（固定为 1）"),
  );
  table.primary_key(Index::create().name("pk_bootstrap_claim").col(Id));
  table.check(("ck_bootstrap_claim_id_positive", Expr::col(Id).gt(0)));
  columns::finish_table(&mut table, backend, "utf8mb4_0900_ai_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  vec![]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
