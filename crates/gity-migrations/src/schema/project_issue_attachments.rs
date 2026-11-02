//! Frozen latest schema: 议题附件.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectIssueAttachments {
  Table,
  Id,
  ProjectIssueId,
  UploadedByUserId,
  FileName,
  ContentType,
  ByteSize,
  StorageKey,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectIssueAttachments::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("议题附件");
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
    columns::big_integer(UploadedByUserId, backend)
      .not_null()
      .comment("上传者用户标识"),
  );
  table.col(
    columns::string(FileName, backend, 255)
      .not_null()
      .comment("文件名称"),
  );
  table.col(columns::string(ContentType, backend, 255).comment("媒体类型"));
  table.col(
    columns::big_integer(ByteSize, backend)
      .not_null()
      .comment("字节大小"),
  );
  table.col(
    columns::string(StorageKey, backend, 512)
      .not_null()
      .comment("对象存储键"),
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
  table.primary_key(Index::create().name("pk_project_issue_attachments").col(Id));
  table.check((
    "ck_project_issue_attachments_id_positive",
    Expr::col(Id).gt(0),
  ));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_issue_attachments_project_issue_id")
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
      .name("fk_project_issue_attachments_uploaded_by_user_id")
      .from(Table, UploadedByUserId)
      .to(super::users::Users::Table, super::users::Users::Id)
      .on_delete(ForeignKeyAction::Restrict)
      .on_update(ForeignKeyAction::NoAction),
  );
  columns::finish_table(&mut table, backend, "utf8mb4_unicode_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use ProjectIssueAttachments::*;
  vec![(
    Index::create()
      .name("ux_project_issue_attachments_1")
      .table(Table)
      .unique()
      .col(StorageKey)
      .to_owned(),
    "保证对象存储键唯一",
  )]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
