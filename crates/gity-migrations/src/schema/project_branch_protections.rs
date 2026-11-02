//! Frozen latest schema: 项目分支保护.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectBranchProtections {
  Table,
  Id,
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
  use ProjectBranchProtections::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("项目分支保护");
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
  table.primary_key(
    Index::create()
      .name("pk_project_branch_protections")
      .col(Id),
  );
  table.check((
    "ck_project_branch_protections_id_positive",
    Expr::col(Id).gt(0),
  ));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_branch_protections_project_id")
      .from(Table, ProjectId)
      .to(
        super::projects::Projects::Table,
        super::projects::Projects::Id,
      )
      .on_delete(ForeignKeyAction::Cascade)
      .on_update(ForeignKeyAction::NoAction),
  );
  columns::finish_table(&mut table, backend, "utf8mb4_unicode_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use ProjectBranchProtections::*;
  vec![(
    Index::create()
      .name("ux_project_branch_protections_1")
      .table(Table)
      .unique()
      .col(ProjectId)
      .col(BranchName)
      .to_owned(),
    "保证项目标识、分支名称唯一",
  )]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
