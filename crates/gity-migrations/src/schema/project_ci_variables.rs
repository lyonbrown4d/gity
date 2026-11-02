//! Frozen latest schema: 项目CI变量.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectCiVariables {
  Table,
  Id,
  ProjectId,
  Key,
  Value,
  Masked,
  Protected,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectCiVariables::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("项目CI变量");
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
    columns::string(Key, backend, 255)
      .not_null()
      .comment("变量键名"),
  );
  table.col(columns::text(Value, backend).not_null().comment("变量值"));
  table.col(
    ColumnDef::new(Masked)
      .integer()
      .not_null()
      .comment("是否掩码"),
  );
  table.col(
    ColumnDef::new(Protected)
      .integer()
      .not_null()
      .comment("是否仅用于受保护分支"),
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
  table.primary_key(Index::create().name("pk_project_ci_variables").col(Id));
  table.check(("ck_project_ci_variables_id_positive", Expr::col(Id).gt(0)));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_ci_variables_project_id")
      .from(Table, ProjectId)
      .to(
        super::projects::Projects::Table,
        super::projects::Projects::Id,
      )
      .on_delete(ForeignKeyAction::Cascade)
      .on_update(ForeignKeyAction::NoAction),
  );
  columns::finish_table(&mut table, backend, "utf8mb4_0900_ai_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use ProjectCiVariables::*;
  vec![
    (
      Index::create()
        .name("ix_project_ci_variables_flags")
        .table(Table)
        .col(Masked)
        .col(Protected)
        .to_owned(),
      "按是否掩码、是否仅用于受保护分支查询",
    ),
    (
      Index::create()
        .name("ux_project_ci_variables_project_key")
        .table(Table)
        .unique()
        .col(ProjectId)
        .col(Key)
        .to_owned(),
      "保证项目标识、变量键名唯一",
    ),
  ]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
