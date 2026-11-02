//! Frozen latest schema: 软件包版本.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectPackageVersions {
  Table,
  Id,
  ProjectPackageId,
  Version,
  Status,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectPackageVersions::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("软件包版本");
  table.col(
    columns::big_integer(Id, backend)
      .not_null()
      .comment("应用生成的正数雪花标识"),
  );
  table.col(
    columns::big_integer(ProjectPackageId, backend)
      .not_null()
      .comment("软件包标识"),
  );
  table.col(
    columns::string(Version, backend, 255)
      .not_null()
      .comment("版本"),
  );
  table.col(
    columns::string(Status, backend, 64)
      .not_null()
      .comment("运行状态"),
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
  table.primary_key(Index::create().name("pk_project_package_versions").col(Id));
  table.check((
    "ck_project_package_versions_id_positive",
    Expr::col(Id).gt(0),
  ));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_package_versions_project_package_id")
      .from(Table, ProjectPackageId)
      .to(
        super::project_packages::ProjectPackages::Table,
        super::project_packages::ProjectPackages::Id,
      )
      .on_delete(ForeignKeyAction::Cascade)
      .on_update(ForeignKeyAction::NoAction),
  );
  columns::finish_table(&mut table, backend, "utf8mb4_unicode_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use ProjectPackageVersions::*;
  vec![
    (
      Index::create()
        .name("ix_project_package_versions_package_status")
        .table(Table)
        .col(ProjectPackageId)
        .col(Status)
        .to_owned(),
      "按软件包标识、运行状态查询",
    ),
    (
      Index::create()
        .name("ux_project_package_versions_package_version")
        .table(Table)
        .unique()
        .col(ProjectPackageId)
        .col(Version)
        .to_owned(),
      "保证软件包标识、版本唯一",
    ),
  ]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
