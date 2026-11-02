//! Frozen latest schema: 项目运行器.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectRunners {
  Table,
  Id,
  ProjectId,
  Name,
  Description,
  Tags,
  Token,
  Status,
  Active,
  LastContactAt,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectRunners::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("项目运行器");
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
    columns::string(Name, backend, 255)
      .not_null()
      .comment("名称"),
  );
  table.col(columns::text(Description, backend).comment("说明"));
  table.col(columns::string(Tags, backend, 255).comment("运行器标签"));
  table.col(
    columns::string(Token, backend, 255)
      .not_null()
      .comment("访问令牌"),
  );
  table.col(
    columns::string(Status, backend, 64)
      .not_null()
      .comment("运行状态"),
  );
  table.col(
    ColumnDef::new(Active)
      .integer()
      .not_null()
      .comment("是否启用"),
  );
  table.col(
    columns::timestamp(LastContactAt, backend)
      .not_null()
      .comment("最后联系时间"),
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
  table.primary_key(Index::create().name("pk_project_runners").col(Id));
  table.check(("ck_project_runners_id_positive", Expr::col(Id).gt(0)));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_runners_project_id")
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
  use ProjectRunners::*;
  vec![
    (
      Index::create()
        .name("ux_project_runners_1")
        .table(Table)
        .unique()
        .col(Token)
        .to_owned(),
      "保证访问令牌唯一",
    ),
    (
      Index::create()
        .name("ix_project_runners_active_contact")
        .table(Table)
        .col(Active)
        .col(LastContactAt)
        .to_owned(),
      "按是否启用、最后联系时间查询",
    ),
    (
      Index::create()
        .name("ix_project_runners_project_status")
        .table(Table)
        .col(ProjectId)
        .col(Status)
        .to_owned(),
      "按项目标识、运行状态查询",
    ),
  ]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
