//! Frozen latest schema: 项目议题.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectIssues {
  Table,
  Id,
  ProjectId,
  Iid,
  AuthorUserId,
  Title,
  Description,
  State,
  CreatedAt,
  UpdatedAt,
  MilestoneId,
  DueDate,
  Weight,
  Confidential,
  ClosedByUserId,
  ClosedAt,
  UpdatedByUserId,
  IssueType,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectIssues::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("项目议题");
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
    columns::big_integer(AuthorUserId, backend)
      .not_null()
      .comment("作者用户标识"),
  );
  table.col(
    columns::string(Title, backend, 255)
      .not_null()
      .comment("标题"),
  );
  table.col(columns::text(Description, backend).comment("说明"));
  table.col(
    columns::string(State, backend, 64)
      .not_null()
      .comment("业务状态"),
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
  table.col(columns::big_integer(MilestoneId, backend).comment("里程碑标识"));
  table.col(ColumnDef::new(DueDate).date().comment("截止日期"));
  table.col(columns::big_integer(Weight, backend).comment("议题权重"));
  table.col(
    columns::big_integer(Confidential, backend)
      .not_null()
      .default(0i64)
      .comment("是否保密"),
  );
  table.col(columns::big_integer(ClosedByUserId, backend).comment("关闭者用户标识"));
  table.col(columns::timestamp(ClosedAt, backend).comment("关闭时间"));
  table.col(columns::big_integer(UpdatedByUserId, backend).comment("更新者用户标识"));
  table.col(
    columns::string(IssueType, backend, 32)
      .not_null()
      .default("issue")
      .comment("议题类型"),
  );
  table.primary_key(Index::create().name("pk_project_issues").col(Id));
  table.check(("ck_project_issues_id_positive", Expr::col(Id).gt(0)));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_issues_project_id")
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
      .name("fk_project_issues_author_user_id")
      .from(Table, AuthorUserId)
      .to(super::users::Users::Table, super::users::Users::Id)
      .on_delete(ForeignKeyAction::Restrict)
      .on_update(ForeignKeyAction::NoAction),
  );
  columns::finish_table(&mut table, backend, "utf8mb4_unicode_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use ProjectIssues::*;
  vec![
    (
      Index::create()
        .name("ix_project_issues_author")
        .table(Table)
        .col(AuthorUserId)
        .to_owned(),
      "按作者用户标识查询",
    ),
    (
      Index::create()
        .name("ix_project_issues_project_state")
        .table(Table)
        .col(ProjectId)
        .col(State)
        .to_owned(),
      "按项目标识、业务状态查询",
    ),
    (
      Index::create()
        .name("ux_project_issues_project_iid")
        .table(Table)
        .unique()
        .col(ProjectId)
        .col(Iid)
        .to_owned(),
      "保证项目标识、项目内编号唯一",
    ),
    (
      Index::create()
        .name("ix_project_issues_milestone")
        .table(Table)
        .col(MilestoneId)
        .to_owned(),
      "按里程碑标识查询",
    ),
  ]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
