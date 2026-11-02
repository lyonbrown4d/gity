//! Frozen latest schema: 项目待办.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectTodos {
  Table,
  Id,
  UserId,
  OrganizationId,
  ProjectId,
  Kind,
  State,
  TargetType,
  TargetId,
  Title,
  Summary,
  ActionUrl,
  CreatedAt,
  UpdatedAt,
  DoneAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectTodos::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("项目待办");
  table.col(
    columns::big_integer(Id, backend)
      .not_null()
      .comment("应用生成的正数雪花标识"),
  );
  table.col(
    columns::big_integer(UserId, backend)
      .not_null()
      .comment("用户标识"),
  );
  table.col(
    columns::big_integer(OrganizationId, backend)
      .not_null()
      .comment("组织标识"),
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
    columns::string(State, backend, 32)
      .not_null()
      .comment("业务状态"),
  );
  table.col(
    columns::string(TargetType, backend, 64)
      .not_null()
      .comment("目标类型"),
  );
  table.col(
    columns::string(TargetId, backend, 128)
      .not_null()
      .comment("目标标识"),
  );
  table.col(
    columns::string(Title, backend, 255)
      .not_null()
      .comment("标题"),
  );
  table.col(columns::text(Summary, backend).not_null().comment("摘要"));
  table.col(
    columns::string(ActionUrl, backend, 512)
      .not_null()
      .comment("操作链接"),
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
  table.col(columns::timestamp(DoneAt, backend).comment("完成时间"));
  table.primary_key(Index::create().name("pk_project_todos").col(Id));
  table.check(("ck_project_todos_id_positive", Expr::col(Id).gt(0)));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_todos_user_id")
      .from(Table, UserId)
      .to(super::users::Users::Table, super::users::Users::Id)
      .on_delete(ForeignKeyAction::Cascade)
      .on_update(ForeignKeyAction::NoAction),
  );
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_todos_organization_id")
      .from(Table, OrganizationId)
      .to(
        super::organizations::Organizations::Table,
        super::organizations::Organizations::Id,
      )
      .on_delete(ForeignKeyAction::Cascade)
      .on_update(ForeignKeyAction::NoAction),
  );
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_todos_project_id")
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
  use ProjectTodos::*;
  vec![
    (
      Index::create()
        .name("ix_project_todos_organization_user")
        .table(Table)
        .col(OrganizationId)
        .col(UserId)
        .to_owned(),
      "按组织标识、用户标识查询",
    ),
    (
      Index::create()
        .name("ix_project_todos_project_user")
        .table(Table)
        .col(ProjectId)
        .col(UserId)
        .to_owned(),
      "按项目标识、用户标识查询",
    ),
    (
      Index::create()
        .name("ix_project_todos_user_state_created")
        .table(Table)
        .col(UserId)
        .col(State)
        .col(CreatedAt)
        .to_owned(),
      "按用户标识、业务状态、创建时间查询",
    ),
    (
      Index::create()
        .name("ux_project_todos_user_kind_target")
        .table(Table)
        .unique()
        .col(UserId)
        .col(Kind)
        .col(TargetType)
        .col(TargetId)
        .to_owned(),
      "保证用户标识、类别、目标类型、目标标识唯一",
    ),
  ]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
