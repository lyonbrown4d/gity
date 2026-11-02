//! Frozen latest schema: 发布附件链接.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectReleaseLinks {
  Table,
  Id,
  ProjectReleaseId,
  Name,
  Url,
  LinkType,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectReleaseLinks::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("发布附件链接");
  table.col(
    columns::big_integer(Id, backend)
      .not_null()
      .comment("应用生成的正数雪花标识"),
  );
  table.col(
    columns::big_integer(ProjectReleaseId, backend)
      .not_null()
      .comment("发布标识"),
  );
  table.col(
    columns::string(Name, backend, 255)
      .not_null()
      .comment("名称"),
  );
  table.col(columns::text(Url, backend).not_null().comment("链接地址"));
  table.col(
    columns::string(LinkType, backend, 64)
      .not_null()
      .comment("链接类型"),
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
  table.primary_key(Index::create().name("pk_project_release_links").col(Id));
  table.check(("ck_project_release_links_id_positive", Expr::col(Id).gt(0)));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_release_links_project_release_id")
      .from(Table, ProjectReleaseId)
      .to(
        super::project_releases::ProjectReleases::Table,
        super::project_releases::ProjectReleases::Id,
      )
      .on_delete(ForeignKeyAction::Cascade)
      .on_update(ForeignKeyAction::NoAction),
  );
  columns::finish_table(&mut table, backend, "utf8mb4_0900_ai_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use ProjectReleaseLinks::*;
  vec![
    (
      Index::create()
        .name("ix_project_release_links_release")
        .table(Table)
        .col(ProjectReleaseId)
        .to_owned(),
      "按发布标识查询",
    ),
    (
      Index::create()
        .name("ix_project_release_links_type")
        .table(Table)
        .col(LinkType)
        .to_owned(),
      "按链接类型查询",
    ),
  ]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
