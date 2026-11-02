//! Frozen latest schema: GitLFS路径锁.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectLfsLocks {
  Table,
  Id,
  ProjectId,
  OwnerUserId,
  Path,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectLfsLocks::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("GitLFS路径锁");
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
    columns::big_integer(OwnerUserId, backend)
      .not_null()
      .comment("所有者用户标识"),
  );
  table.col(
    columns::string(Path, backend, 512)
      .not_null()
      .comment("仓库内路径"),
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
  table.primary_key(Index::create().name("pk_project_lfs_locks").col(Id));
  table.check(("ck_project_lfs_locks_id_positive", Expr::col(Id).gt(0)));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_lfs_locks_project_id")
      .from(Table, ProjectId)
      .to(
        super::projects::Projects::Table,
        super::projects::Projects::Id,
      )
      .on_delete(ForeignKeyAction::Cascade)
      .on_update(ForeignKeyAction::NoAction),
  );
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_lfs_locks_owner_user_id")
      .from(Table, OwnerUserId)
      .to(super::users::Users::Table, super::users::Users::Id)
      .on_delete(ForeignKeyAction::Cascade)
      .on_update(ForeignKeyAction::NoAction),
  );
  columns::finish_table(&mut table, backend, "utf8mb4_unicode_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use ProjectLfsLocks::*;
  vec![
    (
      Index::create()
        .name("ix_project_lfs_locks_owner")
        .table(Table)
        .col(OwnerUserId)
        .to_owned(),
      "按所有者用户标识查询",
    ),
    (
      Index::create()
        .name("ux_project_lfs_locks_project_path")
        .table(Table)
        .unique()
        .col(ProjectId)
        .col(Path)
        .to_owned(),
      "保证项目标识、仓库内路径唯一",
    ),
  ]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
