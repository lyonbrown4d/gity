//! Frozen latest schema: 项目软件包.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectPackages {
  Table,
  Id,
  ProjectId,
  Type,
  Name,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectPackages::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("项目软件包");
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
    columns::string(Type, backend, 64)
      .not_null()
      .comment("类型"),
  );
  table.col(
    columns::string(Name, backend, 255)
      .not_null()
      .comment("名称"),
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
  table.primary_key(Index::create().name("pk_project_packages").col(Id));
  table.check(("ck_project_packages_id_positive", Expr::col(Id).gt(0)));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_packages_project_id")
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
  use ProjectPackages::*;
  vec![(
    Index::create()
      .name("ux_project_packages_project_type_name")
      .table(Table)
      .unique()
      .col(ProjectId)
      .col(Type)
      .col(Name)
      .to_owned(),
    "保证项目标识、类型、名称唯一",
  )]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
