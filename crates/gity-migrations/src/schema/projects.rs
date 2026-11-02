//! Frozen latest schema: 项目.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum Projects {
  Table,
  Id,
  OrganizationId,
  Name,
  PathKey,
  FullPath,
  Visibility,
  Description,
  DefaultBranch,
  Status,
  DeletedAt,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use Projects::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("项目");
  table.col(
    columns::big_integer(Id, backend)
      .not_null()
      .comment("应用生成的正数雪花标识"),
  );
  table.col(
    columns::big_integer(OrganizationId, backend)
      .not_null()
      .comment("组织标识"),
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
  table.col(
    columns::string(Visibility, backend, 64)
      .not_null()
      .comment("可见范围"),
  );
  table.col(columns::text(Description, backend).comment("说明"));
  table.col(
    columns::string(DefaultBranch, backend, 255)
      .not_null()
      .comment("默认分支"),
  );
  table.col(
    columns::string(Status, backend, 64)
      .not_null()
      .comment("运行状态"),
  );
  table.col(columns::timestamp(DeletedAt, backend).comment("删除时间"));
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
  table.primary_key(Index::create().name("pk_projects").col(Id));
  table.check(("ck_projects_id_positive", Expr::col(Id).gt(0)));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_projects_organization_id")
      .from(Table, OrganizationId)
      .to(
        super::organizations::Organizations::Table,
        super::organizations::Organizations::Id,
      )
      .on_delete(ForeignKeyAction::Cascade)
      .on_update(ForeignKeyAction::NoAction),
  );
  columns::finish_table(&mut table, backend, "utf8mb4_unicode_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use Projects::*;
  vec![
    (
      Index::create()
        .name("ux_projects_1")
        .table(Table)
        .unique()
        .col(FullPath)
        .to_owned(),
      "保证完整路径唯一",
    ),
    (
      Index::create()
        .name("ix_projects_organization_status")
        .table(Table)
        .col(OrganizationId)
        .col(Status)
        .to_owned(),
      "按组织标识、运行状态查询",
    ),
    (
      Index::create()
        .name("ix_projects_visibility")
        .table(Table)
        .col(Visibility)
        .to_owned(),
      "按可见范围查询",
    ),
  ]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
