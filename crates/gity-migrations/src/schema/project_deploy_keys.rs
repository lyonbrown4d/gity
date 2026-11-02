//! Frozen latest schema: 项目部署密钥.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

use super::columns;

#[derive(DeriveIden)]
pub(super) enum ProjectDeployKeys {
  Table,
  Id,
  ProjectId,
  Title,
  Fingerprint,
  PublicKey,
  CanPush,
  CreatedByUserId,
  LastUsedAt,
  CreatedAt,
  UpdatedAt,
}

pub(super) fn table(backend: DbBackend) -> TableCreateStatement {
  use ProjectDeployKeys::*;
  let mut table = sea_orm_migration::sea_query::Table::create();
  table.table(Table).comment("项目部署密钥");
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
    columns::string(Title, backend, 255)
      .not_null()
      .comment("标题"),
  );
  table.col(
    columns::string(Fingerprint, backend, 255)
      .not_null()
      .comment("公钥指纹"),
  );
  table.col(
    columns::text(PublicKey, backend)
      .not_null()
      .comment("SSH公钥"),
  );
  table.col(
    ColumnDef::new(CanPush)
      .integer()
      .not_null()
      .comment("允许推送"),
  );
  table.col(
    columns::big_integer(CreatedByUserId, backend)
      .not_null()
      .comment("创建者用户标识"),
  );
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
  table.primary_key(Index::create().name("pk_project_deploy_keys").col(Id));
  table.check(("ck_project_deploy_keys_id_positive", Expr::col(Id).gt(0)));
  table.foreign_key(
    ForeignKey::create()
      .name("fk_project_deploy_keys_project_id")
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
      .name("fk_project_deploy_keys_created_by_user_id")
      .from(Table, CreatedByUserId)
      .to(super::users::Users::Table, super::users::Users::Id)
      .on_delete(ForeignKeyAction::NoAction)
      .on_update(ForeignKeyAction::NoAction),
  );
  columns::finish_table(&mut table, backend, "utf8mb4_0900_ai_ci");
  table.to_owned()
}

pub(super) fn indexes() -> Vec<(IndexCreateStatement, &'static str)> {
  use ProjectDeployKeys::*;
  vec![
    (
      Index::create()
        .name("ix_project_deploy_keys_created_by")
        .table(Table)
        .col(CreatedByUserId)
        .to_owned(),
      "按创建者用户标识查询",
    ),
    (
      Index::create()
        .name("ux_project_deploy_keys_project_fingerprint")
        .table(Table)
        .unique()
        .col(ProjectId)
        .col(Fingerprint)
        .to_owned(),
      "保证项目标识、公钥指纹唯一",
    ),
    (
      Index::create()
        .name("ix_project_deploy_keys_fingerprint")
        .table(Table)
        .col(Fingerprint)
        .to_owned(),
      "按公钥指纹查询",
    ),
  ]
}

pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  super::metadata::create(manager, table(manager.get_database_backend()), indexes()).await
}
