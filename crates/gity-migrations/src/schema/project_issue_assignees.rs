//! Frozen latest schema: 议题指派成员.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectIssueAssignees {
  Table,
  Id,
  ProjectIssueId,
  UserId,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectIssueAssignees::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("议题指派成员");
  table.col(
    columns::big_integer(Id, backend)
      .not_null()
      .comment("应用生成的正数雪花标识"),
  );
  table.col(
    columns::big_integer(ProjectIssueId, backend)
      .not_null()
      .comment("议题标识"),
  );
  table.col(
    columns::big_integer(UserId, backend)
      .not_null()
      .comment("用户标识"),
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
  table.primary_key(Index::create().name("pk_project_issue_assignees").col(Id));
  table.check((
    "ck_project_issue_assignees_id_positive",
    Expr::col(Id).gt(0),
  ));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_issue_assignees_project_issue_id")
      .from(Table, ProjectIssueId)
      .to(
        super::project_issues::ProjectIssues::Table,
        super::project_issues::ProjectIssues::Id,
      )
      .on_delete(ForeignKeyAction::Cascade)
      .on_update(ForeignKeyAction::NoAction),
  );
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_issue_assignees_user_id")
      .from(Table, UserId)
      .to(super::users::Users::Table, super::users::Users::Id)
      .on_delete(ForeignKeyAction::Cascade)
      .on_update(ForeignKeyAction::NoAction),
  );
  columns::finish_table(&mut table, backend, "utf8mb4_unicode_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use ProjectIssueAssignees::*;
  vec![
    (
      Index::create()
        .name("ux_project_issue_assignees_1")
        .table(Table)
        .unique()
        .col(ProjectIssueId)
        .col(UserId)
        .to_owned(),
      "保证议题标识、用户标识唯一",
    ),
    (
      Index::create()
        .name("ix_project_issue_assignees_user")
        .table(Table)
        .col(UserId)
        .to_owned(),
      "按用户标识查询",
    ),
  ]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
