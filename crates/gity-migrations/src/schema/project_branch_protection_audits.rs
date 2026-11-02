//! Frozen latest schema: 分支保护审计快照.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectBranchProtectionAudits {
  Table,
  RevisionId,
  Operation,
  ProjectBranchProtectionId,
  ProjectId,
  BranchName,
  RuleType,
  PushAccessLevel,
  MergeAccessLevel,
  RequireMergeRequest,
  RequirePipelineSuccess,
  AllowForcePush,
  AllowDelete,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectBranchProtectionAudits::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("分支保护审计快照");
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
    columns::big_integer(ProjectBranchProtectionId, backend)
      .not_null()
      .comment("分支保护规则标识"),
  );
  table.col(
    columns::big_integer(ProjectId, backend)
      .not_null()
      .comment("项目标识"),
  );
  table.col(
    columns::string(BranchName, backend, 255)
      .not_null()
      .comment("分支名称"),
  );
  table.col(
    columns::string(RuleType, backend, 64)
      .not_null()
      .comment("保护规则类型"),
  );
  table.col(
    columns::string(PushAccessLevel, backend, 64)
      .not_null()
      .comment("推送权限级别"),
  );
  table.col(
    columns::string(MergeAccessLevel, backend, 64)
      .not_null()
      .comment("合并权限级别"),
  );
  table.col(
    ColumnDef::new(RequireMergeRequest)
      .integer()
      .not_null()
      .comment("要求合并请求"),
  );
  table.col(
    ColumnDef::new(RequirePipelineSuccess)
      .integer()
      .not_null()
      .comment("要求流水线成功"),
  );
  table.col(
    ColumnDef::new(AllowForcePush)
      .integer()
      .not_null()
      .comment("允许强制推送"),
  );
  table.col(
    ColumnDef::new(AllowDelete)
      .integer()
      .not_null()
      .comment("允许删除分支"),
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
      .name("fk_project_branch_protection_audits_revision_id")
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
  use ProjectBranchProtectionAudits::*;
  vec![
    (
      Index::create()
        .name("ix_project_branch_protection_audits_entity")
        .table(Table)
        .col(ProjectBranchProtectionId)
        .to_owned(),
      "按分支保护规则标识查询",
    ),
    (
      Index::create()
        .name("ix_project_branch_protection_audits_project")
        .table(Table)
        .col(ProjectId)
        .to_owned(),
      "按项目标识查询",
    ),
    (
      Index::create()
        .name("ix_project_branch_protection_audits_revision")
        .table(Table)
        .col(RevisionId)
        .to_owned(),
      "按审计修订标识查询",
    ),
  ]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
