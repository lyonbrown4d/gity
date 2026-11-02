//! Frozen latest schema: 项目发布.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectReleases {
  Table,
  Id,
  ProjectId,
  TagName,
  Name,
  Description,
  CreatedByUserId,
  ReleasedAt,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectReleases::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("项目发布");
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
    columns::string(TagName, backend, 255)
      .not_null()
      .comment("Git标签名称"),
  );
  table.col(
    columns::string(Name, backend, 255)
      .not_null()
      .comment("名称"),
  );
  table.col(
    columns::text(Description, backend)
      .not_null()
      .comment("说明"),
  );
  table.col(
    columns::big_integer(CreatedByUserId, backend)
      .not_null()
      .comment("创建者用户标识"),
  );
  table.col(
    columns::timestamp(ReleasedAt, backend)
      .not_null()
      .comment("发布时间"),
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
  table.primary_key(Index::create().name("pk_project_releases").col(Id));
  table.check(("ck_project_releases_id_positive", Expr::col(Id).gt(0)));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_releases_project_id")
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
      .name("fk_project_releases_created_by_user_id")
      .from(Table, CreatedByUserId)
      .to(super::users::Users::Table, super::users::Users::Id)
      .on_delete(ForeignKeyAction::NoAction)
      .on_update(ForeignKeyAction::NoAction),
  );
  columns::finish_table(&mut table, backend, "utf8mb4_0900_ai_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use ProjectReleases::*;
  vec![
    (
      Index::create()
        .name("ix_project_releases_created_by")
        .table(Table)
        .col(CreatedByUserId)
        .to_owned(),
      "按创建者用户标识查询",
    ),
    (
      Index::create()
        .name("ix_project_releases_project_released")
        .table(Table)
        .col(ProjectId)
        .col(ReleasedAt)
        .to_owned(),
      "按项目标识、发布时间查询",
    ),
    (
      Index::create()
        .name("ux_project_releases_project_tag")
        .table(Table)
        .unique()
        .col(ProjectId)
        .col(TagName)
        .to_owned(),
      "保证项目标识、Git标签名称唯一",
    ),
  ]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
