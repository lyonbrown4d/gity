//! Frozen latest schema: 项目访问令牌.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectAccessTokens {
  Table,
  Id,
  ProjectId,
  Kind,
  Name,
  Username,
  Token,
  Scopes,
  CreatedByUserId,
  ExpiresAt,
  RevokedAt,
  LastUsedAt,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectAccessTokens::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("项目访问令牌");
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
    columns::string(Kind, backend, 64)
      .not_null()
      .comment("类别"),
  );
  table.col(
    columns::string(Name, backend, 255)
      .not_null()
      .comment("名称"),
  );
  table.col(
    columns::string(Username, backend, 255)
      .not_null()
      .comment("用户名"),
  );
  table.col(
    columns::string(Token, backend, 255)
      .not_null()
      .comment("访问令牌"),
  );
  table.col(
    columns::text(Scopes, backend)
      .not_null()
      .comment("授权范围"),
  );
  table.col(
    columns::big_integer(CreatedByUserId, backend)
      .not_null()
      .comment("创建者用户标识"),
  );
  table.col(columns::timestamp(ExpiresAt, backend).comment("过期时间"));
  table.col(columns::timestamp(RevokedAt, backend).comment("撤销时间"));
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
  table.primary_key(Index::create().name("pk_project_access_tokens").col(Id));
  table.check(("ck_project_access_tokens_id_positive", Expr::col(Id).gt(0)));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_access_tokens_project_id")
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
      .name("fk_project_access_tokens_created_by_user_id")
      .from(Table, CreatedByUserId)
      .to(super::users::Users::Table, super::users::Users::Id)
      .on_delete(ForeignKeyAction::NoAction)
      .on_update(ForeignKeyAction::NoAction),
  );
  columns::finish_table(&mut table, backend, "utf8mb4_0900_ai_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use ProjectAccessTokens::*;
  vec![
    (
      Index::create()
        .name("ix_project_access_tokens_created_by")
        .table(Table)
        .col(CreatedByUserId)
        .to_owned(),
      "按创建者用户标识查询",
    ),
    (
      Index::create()
        .name("ix_project_access_tokens_project_kind")
        .table(Table)
        .col(ProjectId)
        .col(Kind)
        .to_owned(),
      "按项目标识、类别查询",
    ),
    (
      Index::create()
        .name("ix_project_access_tokens_username")
        .table(Table)
        .col(Username)
        .to_owned(),
      "按用户名查询",
    ),
    (
      Index::create()
        .name("ux_project_access_tokens_token")
        .table(Table)
        .unique()
        .col(Token)
        .to_owned(),
      "保证访问令牌唯一",
    ),
  ]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
