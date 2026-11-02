//! Frozen latest schema: 合并请求参与者.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectMergeRequestParticipants {
  Table,
  Id,
  MergeRequestId,
  UserId,
  Role,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectMergeRequestParticipants::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("合并请求参与者");
  table.col(
    columns::big_integer(Id, backend)
      .not_null()
      .comment("应用生成的正数雪花标识"),
  );
  table.col(
    columns::big_integer(MergeRequestId, backend)
      .not_null()
      .comment("合并请求标识"),
  );
  table.col(
    columns::big_integer(UserId, backend)
      .not_null()
      .comment("用户标识"),
  );
  table.col(
    columns::string(Role, backend, 64)
      .not_null()
      .comment("成员角色"),
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
  table.primary_key(
    Index::create()
      .name("pk_project_merge_request_participants")
      .col(Id),
  );
  table.check((
    "ck_project_merge_request_participants_id_positive",
    Expr::col(Id).gt(0),
  ));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_merge_request_participants_merge_request_id")
      .from(Table, MergeRequestId)
      .to(
        super::project_merge_requests::ProjectMergeRequests::Table,
        super::project_merge_requests::ProjectMergeRequests::Id,
      )
      .on_delete(ForeignKeyAction::Cascade)
      .on_update(ForeignKeyAction::NoAction),
  );
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_merge_request_participants_user_id")
      .from(Table, UserId)
      .to(super::users::Users::Table, super::users::Users::Id)
      .on_delete(ForeignKeyAction::Cascade)
      .on_update(ForeignKeyAction::NoAction),
  );
  columns::finish_table(&mut table, backend, "utf8mb4_unicode_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use ProjectMergeRequestParticipants::*;
  vec![
    (
      Index::create()
        .name("ux_project_merge_request_participants_1")
        .table(Table)
        .unique()
        .col(MergeRequestId)
        .col(UserId)
        .col(Role)
        .to_owned(),
      "保证合并请求标识、用户标识、成员角色唯一",
    ),
    (
      Index::create()
        .name("ix_project_merge_request_participants_user")
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
