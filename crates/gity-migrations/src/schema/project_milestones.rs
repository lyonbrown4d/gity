//! Frozen latest schema: 项目里程碑.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectMilestones {
  Table,
  Id,
  ProjectId,
  Iid,
  Title,
  Description,
  State,
  StartDate,
  DueDate,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectMilestones::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("项目里程碑");
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
    columns::big_integer(Iid, backend)
      .not_null()
      .comment("项目内编号"),
  );
  table.col(
    columns::string(Title, backend, 255)
      .not_null()
      .comment("标题"),
  );
  table.col(columns::text(Description, backend).comment("说明"));
  table.col(
    columns::string(State, backend, 32)
      .not_null()
      .default("active")
      .comment("业务状态"),
  );
  table.col(ColumnDef::new(StartDate).date().comment("开始日期"));
  table.col(ColumnDef::new(DueDate).date().comment("截止日期"));
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
  table.primary_key(Index::create().name("pk_project_milestones").col(Id));
  table.check(("ck_project_milestones_id_positive", Expr::col(Id).gt(0)));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_milestones_project_id")
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
  use ProjectMilestones::*;
  vec![
    (
      Index::create()
        .name("ux_project_milestones_1")
        .table(Table)
        .unique()
        .col(ProjectId)
        .col(Iid)
        .to_owned(),
      "保证项目标识、项目内编号唯一",
    ),
    (
      Index::create()
        .name("ix_project_milestones_project_state")
        .table(Table)
        .col(ProjectId)
        .col(State)
        .to_owned(),
      "按项目标识、业务状态查询",
    ),
  ]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
