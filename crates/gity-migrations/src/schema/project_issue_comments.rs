//! Frozen latest schema: 议题评论.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectIssueComments {
  Table,
  Id,
  ProjectIssueId,
  AuthorUserId,
  Body,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectIssueComments::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("议题评论");
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
    columns::big_integer(AuthorUserId, backend)
      .not_null()
      .comment("作者用户标识"),
  );
  table.col(columns::text(Body, backend).not_null().comment("正文"));
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
  table.primary_key(Index::create().name("pk_project_issue_comments").col(Id));
  table.check(("ck_project_issue_comments_id_positive", Expr::col(Id).gt(0)));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_issue_comments_project_issue_id")
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
      .name("fk_project_issue_comments_author_user_id")
      .from(Table, AuthorUserId)
      .to(super::users::Users::Table, super::users::Users::Id)
      .on_delete(ForeignKeyAction::Restrict)
      .on_update(ForeignKeyAction::NoAction),
  );
  columns::finish_table(&mut table, backend, "utf8mb4_unicode_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use ProjectIssueComments::*;
  vec![(
    Index::create()
      .name("ix_project_issue_comments_issue_created")
      .table(Table)
      .col(ProjectIssueId)
      .col(CreatedAt)
      .to_owned(),
    "按议题标识、创建时间查询",
  )]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
