//! Frozen latest schema: CI变量审计快照.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectCiVariableAudits {
  Table,
  RevisionId,
  Operation,
  ProjectCiVariableId,
  ProjectId,
  Key,
  Masked,
  Protected,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectCiVariableAudits::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("CI变量审计快照");
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
    columns::big_integer(ProjectCiVariableId, backend)
      .not_null()
      .comment("CI变量标识"),
  );
  table.col(
    columns::big_integer(ProjectId, backend)
      .not_null()
      .comment("项目标识"),
  );
  table.col(
    columns::string(Key, backend, 255)
      .not_null()
      .comment("变量键名"),
  );
  table.col(
    ColumnDef::new(Masked)
      .integer()
      .not_null()
      .comment("是否掩码"),
  );
  table.col(
    ColumnDef::new(Protected)
      .integer()
      .not_null()
      .comment("是否仅用于受保护分支"),
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
      .name("fk_project_ci_variable_audits_revision_id")
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
  use ProjectCiVariableAudits::*;
  vec![
    (
      Index::create()
        .name("ix_project_ci_variable_audits_entity")
        .table(Table)
        .col(ProjectCiVariableId)
        .to_owned(),
      "按CI变量标识查询",
    ),
    (
      Index::create()
        .name("ix_project_ci_variable_audits_key")
        .table(Table)
        .col(Key)
        .to_owned(),
      "按变量键名查询",
    ),
    (
      Index::create()
        .name("ix_project_ci_variable_audits_project")
        .table(Table)
        .col(ProjectId)
        .to_owned(),
      "按项目标识查询",
    ),
    (
      Index::create()
        .name("ix_project_ci_variable_audits_revision")
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
