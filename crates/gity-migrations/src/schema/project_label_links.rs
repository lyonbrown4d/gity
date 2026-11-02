//! Frozen latest schema: 项目标签关联.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectLabelLinks {
  Table,
  Id,
  ProjectId,
  LabelId,
  TargetType,
  TargetId,
  CreatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectLabelLinks::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("项目标签关联");
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
    columns::big_integer(LabelId, backend)
      .not_null()
      .comment("标签标识"),
  );
  table.col(
    columns::string(TargetType, backend, 32)
      .not_null()
      .comment("目标类型"),
  );
  table.col(
    columns::big_integer(TargetId, backend)
      .not_null()
      .comment("目标标识"),
  );
  table.col(
    columns::timestamp(CreatedAt, backend)
      .not_null()
      .comment("创建时间"),
  );
  table.primary_key(Index::create().name("pk_project_label_links").col(Id));
  table.check(("ck_project_label_links_id_positive", Expr::col(Id).gt(0)));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_label_links_project_id")
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
      .name("fk_project_label_links_label_id")
      .from(Table, LabelId)
      .to(
        super::project_labels::ProjectLabels::Table,
        super::project_labels::ProjectLabels::Id,
      )
      .on_delete(ForeignKeyAction::Cascade)
      .on_update(ForeignKeyAction::NoAction),
  );
  columns::finish_table(&mut table, backend, "utf8mb4_unicode_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use ProjectLabelLinks::*;
  vec![
    (
      Index::create()
        .name("ux_project_label_links_1")
        .table(Table)
        .unique()
        .col(LabelId)
        .col(TargetType)
        .col(TargetId)
        .to_owned(),
      "保证标签标识、目标类型、目标标识唯一",
    ),
    (
      Index::create()
        .name("ix_project_label_links_target")
        .table(Table)
        .col(TargetType)
        .col(TargetId)
        .to_owned(),
      "按目标类型、目标标识查询",
    ),
  ]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
