//! Frozen latest schema: 合并批准规则审计快照.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectMergeRequestApprovalRuleAudits {
  Table,
  RevisionId,
  Operation,
  ProjectMergeRequestApprovalRuleId,
  ProjectId,
  Name,
  TargetBranch,
  ApprovalsRequired,
  EligibleUserIds,
  CodeOwner,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectMergeRequestApprovalRuleAudits::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("合并批准规则审计快照");
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
    columns::big_integer(ProjectMergeRequestApprovalRuleId, backend)
      .not_null()
      .comment("合并批准规则标识"),
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
  table.col(
    columns::string(TargetBranch, backend, 255)
      .not_null()
      .comment("目标分支"),
  );
  table.col(
    ColumnDef::new(ApprovalsRequired)
      .integer()
      .not_null()
      .comment("所需批准数"),
  );
  table.col(
    columns::text(EligibleUserIds, backend)
      .not_null()
      .comment("可批准用户标识列表"),
  );
  table.col(
    ColumnDef::new(CodeOwner)
      .integer()
      .not_null()
      .comment("代码所有者规则"),
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
      .name("fk_project_merge_request_approval_rule_audits_revision_id")
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
  use ProjectMergeRequestApprovalRuleAudits::*;
  vec![
    (
      Index::create()
        .name("ix_project_mr_approval_rule_audits_entity")
        .table(Table)
        .col(ProjectMergeRequestApprovalRuleId)
        .to_owned(),
      "按合并批准规则标识查询",
    ),
    (
      Index::create()
        .name("ix_project_mr_approval_rule_audits_project")
        .table(Table)
        .col(ProjectId)
        .to_owned(),
      "按项目标识查询",
    ),
    (
      Index::create()
        .name("ix_project_mr_approval_rule_audits_revision")
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
