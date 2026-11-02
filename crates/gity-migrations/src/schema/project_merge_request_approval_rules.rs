//! Frozen latest schema: 合并请求批准规则.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectMergeRequestApprovalRules {
  Table,
  Id,
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
  use ProjectMergeRequestApprovalRules::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("合并请求批准规则");
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
  table.primary_key(
    Index::create()
      .name("pk_project_merge_request_approval_rules")
      .col(Id),
  );
  table.check((
    "ck_project_merge_request_approval_rules_id_positive",
    Expr::col(Id).gt(0),
  ));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_merge_request_approval_rules_project_id")
      .from(Table, ProjectId)
      .to(
        super::projects::Projects::Table,
        super::projects::Projects::Id,
      )
      .on_delete(ForeignKeyAction::Cascade)
      .on_update(ForeignKeyAction::NoAction),
  );
  columns::finish_table(&mut table, backend, "utf8mb4_0900_ai_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use ProjectMergeRequestApprovalRules::*;
  vec![
    (
      Index::create()
        .name("ix_project_mr_approval_rules_code_owner")
        .table(Table)
        .col(CodeOwner)
        .to_owned(),
      "按代码所有者规则查询",
    ),
    (
      Index::create()
        .name("ix_project_mr_approval_rules_project_branch")
        .table(Table)
        .col(ProjectId)
        .col(TargetBranch)
        .to_owned(),
      "按项目标识、目标分支查询",
    ),
  ]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
