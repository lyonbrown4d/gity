//! Frozen latest schema: 合并请求.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectMergeRequests {
  Table,
  Id,
  ProjectId,
  Iid,
  AuthorUserId,
  Title,
  Description,
  State,
  SourceBranch,
  TargetBranch,
  CreatedAt,
  UpdatedAt,
  MilestoneId,
  MergedAt,
  MergedByUserId,
  MergeCommitSha,
  HeadSha,
  ClosedAt,
  Squash,
  Draft,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectMergeRequests::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("合并请求");
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
    columns::string(SourceBranch, backend, 255)
      .not_null()
      .comment("源分支"),
  );
  table.col(
    columns::string(TargetBranch, backend, 255)
      .not_null()
      .comment("目标分支"),
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
  table.col(columns::timestamp(MergedAt, backend).comment("合并时间"));
  table.col(columns::big_integer(MergedByUserId, backend).comment("合并者用户标识"));
  table.col(columns::string(MergeCommitSha, backend, 64).comment("合并提交哈希"));
  table.col(columns::string(HeadSha, backend, 64).comment("差异头提交哈希"));
  table.col(columns::timestamp(ClosedAt, backend).comment("关闭时间"));
  table.col(
    columns::big_integer(Squash, backend)
      .not_null()
      .default(0i64)
      .comment("是否压缩提交"),
  );
  table.col(
    columns::big_integer(Draft, backend)
      .not_null()
      .default(0i64)
      .comment("是否为草稿"),
  );
  table.primary_key(Index::create().name("pk_project_merge_requests").col(Id));
  table.check(("ck_project_merge_requests_id_positive", Expr::col(Id).gt(0)));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_merge_requests_project_id")
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
      .name("fk_project_merge_requests_author_user_id")
      .from(Table, AuthorUserId)
      .to(super::users::Users::Table, super::users::Users::Id)
      .on_delete(ForeignKeyAction::Restrict)
      .on_update(ForeignKeyAction::NoAction),
  );
  columns::finish_table(&mut table, backend, "utf8mb4_unicode_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use ProjectMergeRequests::*;
  vec![
    (
      Index::create()
        .name("ix_project_merge_requests_author")
        .table(Table)
        .col(AuthorUserId)
        .to_owned(),
      "按作者用户标识查询",
    ),
    (
      Index::create()
        .name("ix_project_merge_requests_branches")
        .table(Table)
        .col(ProjectId)
        .col(SourceBranch)
        .col(TargetBranch)
        .to_owned(),
      "按项目标识、源分支、目标分支查询",
    ),
    (
      Index::create()
        .name("ix_project_merge_requests_project_state")
        .table(Table)
        .col(ProjectId)
        .col(State)
        .to_owned(),
      "按项目标识、业务状态查询",
    ),
    (
      Index::create()
        .name("ux_project_merge_requests_project_iid")
        .table(Table)
        .unique()
        .col(ProjectId)
        .col(Iid)
        .to_owned(),
      "保证项目标识、项目内编号唯一",
    ),
    (
      Index::create()
        .name("ix_project_merge_requests_milestone")
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
