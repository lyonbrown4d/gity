//! Frozen latest schema: 部署密钥审计快照.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectDeployKeyAudits {
  Table,
  RevisionId,
  Operation,
  ProjectDeployKeyId,
  ProjectId,
  Title,
  Fingerprint,
  CanPush,
  CreatedByUserId,
  LastUsedAt,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectDeployKeyAudits::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("部署密钥审计快照");
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
    columns::big_integer(ProjectDeployKeyId, backend)
      .not_null()
      .comment("部署密钥标识"),
  );
  table.col(
    columns::big_integer(ProjectId, backend)
      .not_null()
      .comment("项目标识"),
  );
  table.col(
    columns::string(Title, backend, 255)
      .not_null()
      .comment("标题"),
  );
  table.col(
    columns::string(Fingerprint, backend, 255)
      .not_null()
      .comment("公钥指纹"),
  );
  table.col(
    ColumnDef::new(CanPush)
      .integer()
      .not_null()
      .comment("允许推送"),
  );
  table.col(
    columns::big_integer(CreatedByUserId, backend)
      .not_null()
      .comment("创建者用户标识"),
  );
  table.col(columns::timestamp(LastUsedAt, backend).comment("最后使用时间"));
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
      .name("fk_project_deploy_key_audits_revision_id")
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
  use ProjectDeployKeyAudits::*;
  vec![
    (
      Index::create()
        .name("ix_project_deploy_key_audits_entity")
        .table(Table)
        .col(ProjectDeployKeyId)
        .to_owned(),
      "按部署密钥标识查询",
    ),
    (
      Index::create()
        .name("ix_project_deploy_key_audits_fingerprint")
        .table(Table)
        .col(Fingerprint)
        .to_owned(),
      "按公钥指纹查询",
    ),
    (
      Index::create()
        .name("ix_project_deploy_key_audits_project")
        .table(Table)
        .col(ProjectId)
        .to_owned(),
      "按项目标识查询",
    ),
    (
      Index::create()
        .name("ix_project_deploy_key_audits_revision")
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
