//! Frozen latest schema: 项目成员审计快照.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectMemberAudits {
  Table,
  RevisionId,
  Operation,
  ProjectMemberId,
  ProjectId,
  UserId,
  Role,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectMemberAudits::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("项目成员审计快照");
  table.col(
    columns::big_integer(RevisionId, backend)
      .not_null()
      .comment("审计修订标识"),
  );
  table.col(
    columns::string(Operation, backend, 32)
      .not_null()
      .comment("审计操作"),
  );
  table.col(
    columns::big_integer(ProjectMemberId, backend)
      .not_null()
      .comment("项目成员标识"),
  );
  table.col(
    columns::big_integer(ProjectId, backend)
      .not_null()
      .comment("项目标识"),
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
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_member_audits_revision_id")
      .from(Table, RevisionId)
      .to(
        super::audit_revisions::AuditRevisions::Table,
        super::audit_revisions::AuditRevisions::Id,
      )
      .on_delete(ForeignKeyAction::NoAction)
      .on_update(ForeignKeyAction::NoAction),
  );
  columns::finish_table(&mut table, backend, "utf8mb4_unicode_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use ProjectMemberAudits::*;
  vec![
    (
      Index::create()
        .name("ix_project_member_audits_entity")
        .table(Table)
        .col(ProjectMemberId)
        .to_owned(),
      "按项目成员标识查询",
    ),
    (
      Index::create()
        .name("ix_project_member_audits_project")
        .table(Table)
        .col(ProjectId)
        .to_owned(),
      "按项目标识查询",
    ),
    (
      Index::create()
        .name("ix_project_member_audits_revision")
        .table(Table)
        .col(RevisionId)
        .to_owned(),
      "按审计修订标识查询",
    ),
    (
      Index::create()
        .name("ix_project_member_audits_user")
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
