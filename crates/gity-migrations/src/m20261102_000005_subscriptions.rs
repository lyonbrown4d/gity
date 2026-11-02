//! Frozen issue/merge-request subscription and notification preference schema.
//!
//! Subscriptions add notification recipients beyond the existing participants;
//! preferences let a user mute one notification kind without losing the row.
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum ProjectSubscriptions {
  Table,
  Id,
  ProjectId,
  UserId,
  TargetType,
  TargetId,
  CreatedAt,
}

#[derive(DeriveIden)]
enum UserNotificationPreferences {
  Table,
  Id,
  UserId,
  Kind,
  Enabled,
  CreatedAt,
  UpdatedAt,
}

#[derive(DeriveIden)]
enum Projects {
  Table,
  Id,
}

#[derive(DeriveIden)]
enum Users {
  Table,
  Id,
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
    // SQLite INTEGER PRIMARY KEY aliases ROWID; BIGINT keeps application IDs explicit.
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

fn string(name: impl IntoIden, length: u32) -> ColumnDef {
  ColumnDef::new(name)
    .string_len(length)
    .not_null()
    .to_owned()
}

fn mysql_storage(table: &mut TableCreateStatement) {
  table
    .engine("InnoDB")
    .character_set("utf8mb4")
    .collate("utf8mb4_bin");
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    use ProjectSubscriptions as Subscriptions;
    use UserNotificationPreferences as Preferences;
    let backend = manager.get_database_backend();

    let mut subscriptions = sea_orm_migration::sea_query::Table::create();
    subscriptions
      .table(Subscriptions::Table)
      .comment("Issue and merge request subscriptions")
      .col(
        identifier(Subscriptions::Id, backend)
          .not_null()
          .primary_key()
          .comment("Application snowflake ID"),
      )
      .col(
        identifier(Subscriptions::ProjectId, backend)
          .not_null()
          .comment("Owning project ID"),
      )
      .col(
        identifier(Subscriptions::UserId, backend)
          .not_null()
          .comment("Subscribing user ID"),
      )
      .col(
        ColumnDef::new(Subscriptions::TargetType)
          .string_len(32)
          .not_null()
          .comment("issue or merge_request"),
      )
      .col(
        identifier(Subscriptions::TargetId, backend)
          .not_null()
          .comment("Issue or merge request ID"),
      )
      .col(
        timestamp(Subscriptions::CreatedAt, backend)
          .not_null()
          .comment("Creation time"),
      )
      .check((
        "ck_project_subscriptions_id_positive",
        Expr::col(Subscriptions::Id).gt(0),
      ))
      .foreign_key(
        ForeignKey::create()
          .name("fk_project_subscriptions_project_id")
          .from(Subscriptions::Table, Subscriptions::ProjectId)
          .to(Projects::Table, Projects::Id)
          .on_delete(ForeignKeyAction::Cascade)
          .on_update(ForeignKeyAction::NoAction),
      )
      .foreign_key(
        ForeignKey::create()
          .name("fk_project_subscriptions_user_id")
          .from(Subscriptions::Table, Subscriptions::UserId)
          .to(Users::Table, Users::Id)
          .on_delete(ForeignKeyAction::Cascade)
          .on_update(ForeignKeyAction::NoAction),
      );
    if backend == sea_orm::DbBackend::MySql {
      mysql_storage(&mut subscriptions);
    }
    manager.create_table(subscriptions.to_owned()).await?;
    manager
      .create_index(
        Index::create()
          .name("ux_project_subscriptions_user_target")
          .table(Subscriptions::Table)
          .col(Subscriptions::UserId)
          .col(Subscriptions::TargetType)
          .col(Subscriptions::TargetId)
          .unique()
          .to_owned(),
      )
      .await?;
    manager
      .create_index(
        Index::create()
          .name("ix_project_subscriptions_project_target")
          .table(Subscriptions::Table)
          .col(Subscriptions::ProjectId)
          .col(Subscriptions::TargetType)
          .col(Subscriptions::TargetId)
          .to_owned(),
      )
      .await?;

    let mut preferences = sea_orm_migration::sea_query::Table::create();
    preferences
      .table(Preferences::Table)
      .comment("Per-user notification preferences by kind")
      .col(
        identifier(Preferences::Id, backend)
          .not_null()
          .primary_key()
          .comment("Application snowflake ID"),
      )
      .col(
        identifier(Preferences::UserId, backend)
          .not_null()
          .comment("Owning user ID"),
      )
      .col(string(Preferences::Kind, 64).comment("Notification kind"))
      .col(
        ColumnDef::new(Preferences::Enabled)
          .boolean()
          .not_null()
          .comment("False mutes the kind"),
      )
      .col(
        timestamp(Preferences::CreatedAt, backend)
          .not_null()
          .comment("Creation time"),
      )
      .col(
        timestamp(Preferences::UpdatedAt, backend)
          .not_null()
          .comment("Last change time"),
      )
      .check((
        "ck_user_notification_preferences_id_positive",
        Expr::col(Preferences::Id).gt(0),
      ))
      .foreign_key(
        ForeignKey::create()
          .name("fk_user_notification_preferences_user_id")
          .from(Preferences::Table, Preferences::UserId)
          .to(Users::Table, Users::Id)
          .on_delete(ForeignKeyAction::Cascade)
          .on_update(ForeignKeyAction::NoAction),
      );
    if backend == sea_orm::DbBackend::MySql {
      mysql_storage(&mut preferences);
    }
    manager.create_table(preferences.to_owned()).await?;
    manager
      .create_index(
        Index::create()
          .name("ux_user_notification_preferences_user_kind")
          .table(Preferences::Table)
          .col(Preferences::UserId)
          .col(Preferences::Kind)
          .unique()
          .to_owned(),
      )
      .await?;

    if backend == sea_orm::DbBackend::Postgres {
      // SeaQuery's PostgreSQL CREATE TABLE omits comments; these fixed DDL
      // statements keep schema documentation without application SQL.
      let connection = manager.get_connection();
      for statement in [
        "COMMENT ON TABLE project_subscriptions IS 'Issue and merge request subscriptions'",
        "COMMENT ON COLUMN project_subscriptions.id IS 'Application snowflake ID'",
        "COMMENT ON COLUMN project_subscriptions.project_id IS 'Owning project ID'",
        "COMMENT ON COLUMN project_subscriptions.user_id IS 'Subscribing user ID'",
        "COMMENT ON COLUMN project_subscriptions.target_type IS 'issue or merge_request'",
        "COMMENT ON COLUMN project_subscriptions.target_id IS 'Issue or merge request ID'",
        "COMMENT ON COLUMN project_subscriptions.created_at IS 'Creation time'",
        "COMMENT ON INDEX ux_project_subscriptions_user_target IS 'One subscription per user and target'",
        "COMMENT ON INDEX ix_project_subscriptions_project_target IS 'Notification fan-out lookup by target'",
        "COMMENT ON TABLE user_notification_preferences IS 'Per-user notification preferences by kind'",
        "COMMENT ON COLUMN user_notification_preferences.id IS 'Application snowflake ID'",
        "COMMENT ON COLUMN user_notification_preferences.user_id IS 'Owning user ID'",
        "COMMENT ON COLUMN user_notification_preferences.kind IS 'Notification kind'",
        "COMMENT ON COLUMN user_notification_preferences.enabled IS 'False mutes the kind'",
        "COMMENT ON COLUMN user_notification_preferences.created_at IS 'Creation time'",
        "COMMENT ON COLUMN user_notification_preferences.updated_at IS 'Last change time'",
        "COMMENT ON INDEX ux_user_notification_preferences_user_kind IS 'One preference row per user and kind'",
      ] {
        connection.execute_unprepared(statement).await?;
      }
    }
    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(
        Table::drop()
          .table(UserNotificationPreferences::Table)
          .to_owned(),
      )
      .await?;
    manager
      .drop_table(Table::drop().table(ProjectSubscriptions::Table).to_owned())
      .await
  }
}
