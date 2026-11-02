//! Frozen latest schema: Git仓库节点放置.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum GitRepositoryPlacements {
  Table,
  ProjectId,
  TransitionTarget,
  PrimaryNode,
  Epoch,
  Generation,
  PendingWrites,
  State,
  ReplicasJson,
  Version,
  LastError,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use GitRepositoryPlacements::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("Git仓库节点放置");
  table.col(
    columns::big_integer(ProjectId, backend)
      .not_null()
      .comment("项目标识"),
  );
  table.col(columns::string(TransitionTarget, backend, 255).comment("迁移目标节点"));
  table.col(
    columns::string(PrimaryNode, backend, 255)
      .not_null()
      .comment("主节点名称"),
  );
  table.col(
    columns::big_integer(Epoch, backend)
      .not_null()
      .comment("主节点任期"),
  );
  table.col(
    columns::big_integer(Generation, backend)
      .not_null()
      .comment("仓库代次"),
  );
  table.col(
    columns::big_integer(PendingWrites, backend)
      .not_null()
      .comment("待完成写入数"),
  );
  table.col(
    columns::string(State, backend, 255)
      .not_null()
      .comment("业务状态"),
  );
  table.col(
    columns::text(ReplicasJson, backend)
      .not_null()
      .comment("副本配置JSON"),
  );
  table.col(
    columns::big_integer(Version, backend)
      .not_null()
      .comment("版本"),
  );
  table.col(columns::string(LastError, backend, 255).comment("最近错误说明"));
  table.col(
    columns::timestamp(UpdatedAt, backend)
      .not_null()
      .comment("更新时间"),
  );
  table.primary_key(
    Index::create()
      .name("pk_git_repository_placements")
      .col(ProjectId),
  );
  table.check((
    "ck_git_repository_placements_project_id_positive",
    Expr::col(ProjectId).gt(0),
  ));
  table.check(("ck_git_placement_pending", Expr::col(PendingWrites).gte(0)));
  columns::finish_table(&mut table, backend, "utf8mb4_unicode_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  vec![]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
