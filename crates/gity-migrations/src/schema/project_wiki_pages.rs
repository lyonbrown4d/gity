//! Frozen latest schema: 项目Wiki页面.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectWikiPages {
  Table,
  Id,
  ProjectId,
  Slug,
  Title,
  Content,
  Format,
  AuthorUserId,
  LastEditedByUserId,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectWikiPages::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("项目Wiki页面");
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
    columns::string(Slug, backend, 255)
      .not_null()
      .comment("页面路径键"),
  );
  table.col(
    columns::string(Title, backend, 255)
      .not_null()
      .comment("标题"),
  );
  table.col(columns::text(Content, backend).comment("内容"));
  table.col(
    columns::string(Format, backend, 64)
      .not_null()
      .comment("内容格式"),
  );
  table.col(
    columns::big_integer(AuthorUserId, backend)
      .not_null()
      .comment("作者用户标识"),
  );
  table.col(
    columns::big_integer(LastEditedByUserId, backend)
      .not_null()
      .comment("最后编辑者用户标识"),
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
  table.primary_key(Index::create().name("pk_project_wiki_pages").col(Id));
  table.check(("ck_project_wiki_pages_id_positive", Expr::col(Id).gt(0)));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_wiki_pages_project_id")
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
      .name("fk_project_wiki_pages_author_user_id")
      .from(Table, AuthorUserId)
      .to(super::users::Users::Table, super::users::Users::Id)
      .on_delete(ForeignKeyAction::Restrict)
      .on_update(ForeignKeyAction::NoAction),
  );
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_wiki_pages_last_edited_by_user_id")
      .from(Table, LastEditedByUserId)
      .to(super::users::Users::Table, super::users::Users::Id)
      .on_delete(ForeignKeyAction::Restrict)
      .on_update(ForeignKeyAction::NoAction),
  );
  columns::finish_table(&mut table, backend, "utf8mb4_unicode_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use ProjectWikiPages::*;
  vec![
    (
      Index::create()
        .name("ux_project_wiki_pages_1")
        .table(Table)
        .unique()
        .col(ProjectId)
        .col(Slug)
        .to_owned(),
      "保证项目标识、页面路径键唯一",
    ),
    (
      Index::create()
        .name("ix_project_wiki_pages_project_format")
        .table(Table)
        .col(ProjectId)
        .col(Format)
        .to_owned(),
      "按项目标识、内容格式查询",
    ),
  ]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
