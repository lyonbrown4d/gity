//! Frozen personal SSH credential schema; public material only.
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum UserSshKeys {
  Table,
  Id,
  UserId,
  Title,
  Fingerprint,
  PublicKey,
  LastUsedAt,
  CreatedAt,
  UpdatedAt,
}

#[derive(DeriveIden)]
enum NativeType {
  #[sea_orm(iden = "BIGINT")]
  SqliteBigInt,
  #[sea_orm(iden = "DATETIME(6)")]
  MySqlDateTime,
}
fn identifier(name: impl IntoIden, backend: sea_orm::DbBackend) -> ColumnDef {
  let mut column = ColumnDef::new(name);
  if backend == sea_orm::DbBackend::Sqlite {
    column.custom(NativeType::SqliteBigInt);
  } else {
    column.big_integer();
  }
  column
}
fn timestamp(name: impl IntoIden, backend: sea_orm::DbBackend) -> ColumnDef {
  let mut column = ColumnDef::new(name);
  if backend == sea_orm::DbBackend::MySql {
    column.custom(NativeType::MySqlDateTime);
  } else {
    column.timestamp();
  }
  column
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    use UserSshKeys::*;
    let backend = manager.get_database_backend();
    let mut table = sea_orm_migration::sea_query::Table::create();
    table
      .table(Table)
      .comment("个人SSH公钥，按当前用户权限授权Git操作")
      .col(
        identifier(Id, backend)
          .not_null()
          .primary_key()
          .comment("应用生成的正数雪花标识"),
      )
      .col(
        identifier(UserId, backend)
          .not_null()
          .comment("所属用户标识"),
      )
      .col(
        ColumnDef::new(Title)
          .string_len(255)
          .not_null()
          .comment("标题"),
      )
      .col(
        ColumnDef::new(Fingerprint)
          .string_len(255)
          .not_null()
          .comment("全局唯一SHA256公钥指纹"),
      )
      .col(
        ColumnDef::new(PublicKey)
          .text()
          .not_null()
          .comment("规范化OpenSSH公钥，不保存私钥"),
      )
      .col(timestamp(LastUsedAt, backend).comment("最后授权Git命令时间"))
      .col(timestamp(CreatedAt, backend).not_null().comment("创建时间"))
      .col(timestamp(UpdatedAt, backend).not_null().comment("更新时间"))
      .check(("ck_user_ssh_keys_id_positive", Expr::col(Id).gt(0)));
    if manager.get_database_backend() == sea_orm::DbBackend::MySql {
      table
        .engine("InnoDB")
        .character_set("utf8mb4")
        .collate("utf8mb4_bin");
    }
    manager.create_table(table.to_owned()).await?;
    manager
      .create_index(
        Index::create()
          .name("ux_user_ssh_keys_fingerprint")
          .table(Table)
          .col(Fingerprint)
          .unique()
          .to_owned(),
      )
      .await?;
    manager
      .create_index(
        Index::create()
          .name("ix_user_ssh_keys_user_id")
          .table(Table)
          .col(UserId)
          .to_owned(),
      )
      .await?;
    if manager.get_database_backend() == sea_orm::DbBackend::Postgres {
      // SeaQuery's PostgreSQL CREATE TABLE omits comments; these fixed DDL statements
      // preserve schema documentation without introducing application SQL.
      let connection = manager.get_connection();
      for statement in [
        "COMMENT ON INDEX ux_user_ssh_keys_fingerprint IS '保证个人SSH公钥指纹全局唯一'",
        "COMMENT ON INDEX ix_user_ssh_keys_user_id IS '按所属用户查询个人SSH公钥'",
        "COMMENT ON TABLE user_ssh_keys IS '个人SSH公钥，按当前用户权限授权Git操作'",
        "COMMENT ON COLUMN user_ssh_keys.id IS '应用生成的正数雪花标识'",
        "COMMENT ON COLUMN user_ssh_keys.user_id IS '所属用户标识'",
        "COMMENT ON COLUMN user_ssh_keys.title IS '标题'",
        "COMMENT ON COLUMN user_ssh_keys.fingerprint IS '全局唯一SHA256公钥指纹'",
        "COMMENT ON COLUMN user_ssh_keys.public_key IS '规范化OpenSSH公钥，不保存私钥'",
        "COMMENT ON COLUMN user_ssh_keys.last_used_at IS '最后授权Git命令时间'",
        "COMMENT ON COLUMN user_ssh_keys.created_at IS '创建时间'",
        "COMMENT ON COLUMN user_ssh_keys.updated_at IS '更新时间'",
      ] {
        connection.execute_unprepared(statement).await?;
      }
    }
    Ok(())
  }
  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(UserSshKeys::Table).to_owned())
      .await
  }
}
