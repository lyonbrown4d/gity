//! Frozen latest schema: 项目制品仓库设置.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectRegistrySettings {
  Table,
  ProjectId,
  Mode,
  MirrorAlias,
  MirrorRepositoryPrefix,
  UpdatedBy,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectRegistrySettings::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("项目制品仓库设置");
  table.col(
    columns::big_integer(ProjectId, backend)
      .not_null()
      .comment("项目标识"),
  );
  table.col(
    columns::string(Mode, backend, 255)
      .not_null()
      .comment("运行模式"),
  );
  table.col(columns::string(MirrorAlias, backend, 255).comment("上游镜像别名"));
  table.col(
    columns::string(MirrorRepositoryPrefix, backend, 255)
      .not_null()
      .comment("上游仓库前缀"),
  );
  table.col(columns::big_integer(UpdatedBy, backend).comment("更新者标识"));
  table.col(
    columns::timestamp(UpdatedAt, backend)
      .not_null()
      .comment("更新时间"),
  );
  table.primary_key(
    Index::create()
      .name("pk_project_registry_settings")
      .col(ProjectId),
  );
  table.check((
    "ck_project_registry_settings_project_id_positive",
    Expr::col(ProjectId).gt(0),
  ));
  columns::finish_table(&mut table, backend, "utf8mb4_unicode_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  vec![]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
