//! Frozen latest schema: NPM分发标签.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectNpmDistTags {
  Table,
  Id,
  ProjectPackageId,
  Tag,
  Version,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectNpmDistTags::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("NPM分发标签");
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
    columns::string(Tag, backend, 128)
      .not_null()
      .comment("分发标签"),
  );
  table.col(
    columns::string(Version, backend, 255)
      .not_null()
      .comment("版本"),
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
  table.primary_key(Index::create().name("pk_project_npm_dist_tags").col(Id));
  table.check(("ck_project_npm_dist_tags_id_positive", Expr::col(Id).gt(0)));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_npm_dist_tags_project_package_id")
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
  use ProjectNpmDistTags::*;
  vec![(
    Index::create()
      .name("ux_project_npm_dist_tags_1")
      .table(Table)
      .unique()
      .col(ProjectPackageId)
      .col(Tag)
      .to_owned(),
    "保证软件包标识、分发标签唯一",
  )]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
