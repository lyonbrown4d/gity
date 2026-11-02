//! Frozen latest schema: 项目审计事件.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectAuditEvents {
  Table,
  Id,
  ProjectId,
  OrganizationId,
  EventName,
  Action,
  ActorUserId,
  TargetType,
  TargetId,
  Summary,
  Payload,
  CreatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectAuditEvents::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("项目审计事件");
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
    columns::big_integer(OrganizationId, backend)
      .not_null()
      .comment("组织标识"),
  );
  table.col(
    columns::string(EventName, backend, 255)
      .not_null()
      .comment("事件名称"),
  );
  table.col(
    columns::string(Action, backend, 64)
      .not_null()
      .comment("动作"),
  );
  table.col(
    columns::big_integer(ActorUserId, backend)
      .not_null()
      .comment("操作者用户标识"),
  );
  table.col(
    columns::string(TargetType, backend, 255)
      .not_null()
      .comment("目标类型"),
  );
  table.col(
    columns::string(TargetId, backend, 255)
      .not_null()
      .comment("目标标识"),
  );
  table.col(
    columns::string(Summary, backend, 255)
      .not_null()
      .comment("摘要"),
  );
  table.col(
    columns::text(Payload, backend)
      .not_null()
      .comment("任务或事件载荷"),
  );
  table.col(
    columns::timestamp(CreatedAt, backend)
      .not_null()
      .comment("创建时间"),
  );
  table.primary_key(Index::create().name("pk_project_audit_events").col(Id));
  table.check(("ck_project_audit_events_id_positive", Expr::col(Id).gt(0)));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_audit_events_project_id")
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
      .name("fk_project_audit_events_organization_id")
      .from(Table, OrganizationId)
      .to(
        super::organizations::Organizations::Table,
        super::organizations::Organizations::Id,
      )
      .on_delete(ForeignKeyAction::Cascade)
      .on_update(ForeignKeyAction::NoAction),
  );
  columns::finish_table(&mut table, backend, "utf8mb4_unicode_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use ProjectAuditEvents::*;
  vec![
    (
      Index::create()
        .name("ix_project_audit_events_actor")
        .table(Table)
        .col(ActorUserId)
        .to_owned(),
      "按操作者用户标识查询",
    ),
    (
      Index::create()
        .name("ix_project_audit_events_event_action")
        .table(Table)
        .col(EventName)
        .col(Action)
        .to_owned(),
      "按事件名称、动作查询",
    ),
    (
      Index::create()
        .name("ix_project_audit_events_project_created")
        .table(Table)
        .col(ProjectId)
        .col(CreatedAt)
        .to_owned(),
      "按项目标识、创建时间查询",
    ),
  ]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
