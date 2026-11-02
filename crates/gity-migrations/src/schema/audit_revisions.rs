//! Frozen latest schema: 审计修订.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum AuditRevisions {
  Table,
  Id,
  CreatedAt,
  Actor,
  Tenant,
  Reason,
  Metadata,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use AuditRevisions::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("审计修订");
  table.col(
    columns::big_integer(Id, backend)
      .not_null()
      .comment("应用生成的正数雪花标识"),
  );
  table.col(
    columns::timestamp(CreatedAt, backend)
      .not_null()
      .comment("创建时间"),
  );
  table.col(columns::string(Actor, backend, 255).comment("操作者"));
  table.col(columns::string(Tenant, backend, 255).comment("租户"));
  table.col(columns::string(Reason, backend, 255).comment("修订原因"));
  table.col(columns::text(Metadata, backend).comment("元数据"));
  table.primary_key(Index::create().name("pk_audit_revisions").col(Id));
  table.check(("ck_audit_revisions_id_positive", Expr::col(Id).gt(0)));
  columns::finish_table(&mut table, backend, "utf8mb4_unicode_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use AuditRevisions::*;
  vec![
    (
      Index::create()
        .name("ix_audit_revisions_actor")
        .table(Table)
        .col(Actor)
        .to_owned(),
      "按操作者查询",
    ),
    (
      Index::create()
        .name("ix_audit_revisions_created_at")
        .table(Table)
        .col(CreatedAt)
        .to_owned(),
      "按创建时间查询",
    ),
  ]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
