//! Frozen latest schema: 项目内编号计数器.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectIidCounters {
  Table,
  ProjectId,
  CounterName,
  CurrentValue,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectIidCounters::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("项目内编号计数器");
  table.col(
    columns::big_integer(ProjectId, backend)
      .not_null()
      .comment("项目标识"),
  );
  table.col(
    columns::string(CounterName, backend, 64)
      .not_null()
      .comment("计数器名称"),
  );
  table.col(
    columns::big_integer(CurrentValue, backend)
      .not_null()
      .default(0i64)
      .comment("当前计数值"),
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
  table.primary_key(
    Index::create()
      .name("pk_project_iid_counters")
      .col(ProjectId)
      .col(CounterName),
  );
  table.check((
    "ck_project_iid_counters_project_id_positive",
    Expr::col(ProjectId).gt(0),
  ));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_iid_counters_project_id")
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
  vec![]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
