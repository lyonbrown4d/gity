//! Frozen latest schema: 组织成员.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum OrganizationMembers {
  Table,
  Id,
  OrganizationId,
  UserId,
  Role,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use OrganizationMembers::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("组织成员");
  table.col(
    columns::big_integer(Id, backend)
      .not_null()
      .comment("应用生成的正数雪花标识"),
  );
  table.col(
    columns::big_integer(OrganizationId, backend)
      .not_null()
      .comment("组织标识"),
  );
  table.col(
    columns::big_integer(UserId, backend)
      .not_null()
      .comment("用户标识"),
  );
  table.col(
    columns::string(Role, backend, 64)
      .not_null()
      .comment("成员角色"),
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
  table.primary_key(Index::create().name("pk_organization_members").col(Id));
  table.check(("ck_organization_members_id_positive", Expr::col(Id).gt(0)));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_organization_members_organization_id")
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
      .name("fk_organization_members_user_id")
      .from(Table, UserId)
      .to(super::users::Users::Table, super::users::Users::Id)
      .on_delete(ForeignKeyAction::Cascade)
      .on_update(ForeignKeyAction::NoAction),
  );
  columns::finish_table(&mut table, backend, "utf8mb4_unicode_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use OrganizationMembers::*;
  vec![
    (
      Index::create()
        .name("ix_organization_members_user")
        .table(Table)
        .col(UserId)
        .to_owned(),
      "按用户标识查询",
    ),
    (
      Index::create()
        .name("ux_organization_members_org_user")
        .table(Table)
        .unique()
        .col(OrganizationId)
        .col(UserId)
        .to_owned(),
      "保证组织标识、用户标识唯一",
    ),
  ]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
