//! Frozen discussion-thread schema plus the inline anchors on existing comments.
//!
//! A discussion groups comments on one issue or merge request. `position` keeps
//! the inline review anchor as JSON text, so no file/line table is introduced.
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum ProjectDiscussions {
  Table,
  Id,
  ProjectId,
  TargetType,
  TargetId,
  Resolved,
  ResolvedByUserId,
  CreatedAt,
  UpdatedAt,
}

#[derive(DeriveIden)]
enum ProjectIssueComments {
  Table,
  DiscussionId,
  Position,
}

#[derive(DeriveIden)]
enum ProjectMergeRequestComments {
  Table,
  DiscussionId,
  Position,
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
    use ProjectDiscussions as Discussions;
    let backend = manager.get_database_backend();

    let mut table = sea_orm_migration::sea_query::Table::create();
    table
      .table(Discussions::Table)
      .comment("Comment threads on one issue or merge request")
      .col(
        identifier(Discussions::Id, backend)
          .not_null()
          .primary_key()
          .comment("Application snowflake ID"),
      )
      .col(
        identifier(Discussions::ProjectId, backend)
          .not_null()
          .comment("Owning project ID"),
      )
      .col(string(Discussions::TargetType, 32).comment("issue or merge_request"))
      .col(
        identifier(Discussions::TargetId, backend)
          .not_null()
          .comment("Issue or merge request ID"),
      )
      .col(
        ColumnDef::new(Discussions::Resolved)
          .boolean()
          .not_null()
          .default(false)
          .comment("Thread resolved state"),
      )
      .col(
        identifier(Discussions::ResolvedByUserId, backend)
          .null()
          .comment("User that resolved the thread"),
      )
      .col(
        timestamp(Discussions::CreatedAt, backend)
          .not_null()
          .comment("Creation time"),
      )
      .col(
        timestamp(Discussions::UpdatedAt, backend)
          .not_null()
          .comment("Last change time"),
      )
      .check((
        "ck_project_discussions_id_positive",
        Expr::col(Discussions::Id).gt(0),
      ))
      .foreign_key(
        ForeignKey::create()
          .name("fk_project_discussions_project_id")
          .from(Discussions::Table, Discussions::ProjectId)
          .to(Projects::Table, Projects::Id)
          .on_delete(ForeignKeyAction::Cascade)
          .on_update(ForeignKeyAction::NoAction),
      )
      .foreign_key(
        ForeignKey::create()
          .name("fk_project_discussions_resolved_by_user_id")
          .from(Discussions::Table, Discussions::ResolvedByUserId)
          .to(Users::Table, Users::Id)
          .on_delete(ForeignKeyAction::SetNull)
          .on_update(ForeignKeyAction::NoAction),
      );
    if backend == sea_orm::DbBackend::MySql {
      mysql_storage(&mut table);
    }
    manager.create_table(table.clone()).await?;
    manager
      .create_index(
        Index::create()
          .name("ix_project_discussions_project_target")
          .table(Discussions::Table)
          .col(Discussions::ProjectId)
          .col(Discussions::TargetType)
          .col(Discussions::TargetId)
          .to_owned(),
      )
      .await?;

    // SQLite rejects several alter options in one statement, so each column is
    // added on its own; PostgreSQL and MySQL accept the same sequence.
    manager
      .alter_table(
        sea_query::Table::alter()
          .table(ProjectIssueComments::Table)
          .add_column(identifier(ProjectIssueComments::DiscussionId, backend).null())
          .to_owned(),
      )
      .await?;
    manager
      .alter_table(
        sea_query::Table::alter()
          .table(ProjectIssueComments::Table)
          .add_column(ColumnDef::new(ProjectIssueComments::Position).text().null())
          .to_owned(),
      )
      .await?;
    manager
      .alter_table(
        sea_query::Table::alter()
          .table(ProjectMergeRequestComments::Table)
          .add_column(identifier(ProjectMergeRequestComments::DiscussionId, backend).null())
          .to_owned(),
      )
      .await?;
    manager
      .alter_table(
        sea_query::Table::alter()
          .table(ProjectMergeRequestComments::Table)
          .add_column(
            ColumnDef::new(ProjectMergeRequestComments::Position)
              .text()
              .null(),
          )
          .to_owned(),
      )
      .await?;

    if backend == sea_orm::DbBackend::Postgres {
      // SeaQuery cannot express COMMENT ON; only these fixed descriptions are raw.
      let connection = manager.get_connection();
      for column in table.get_columns() {
        let description = column
          .get_column_spec()
          .comment
          .as_ref()
          .unwrap()
          .replace('\'', "''");
        connection
          .execute_unprepared(&format!(
            "COMMENT ON COLUMN project_discussions.\"{}\" IS '{}'",
            column.get_column_name(),
            description
          ))
          .await?;
      }
      for statement in [
        "COMMENT ON INDEX ix_project_discussions_project_target IS 'Thread lookup by target'",
        "COMMENT ON COLUMN project_issue_comments.discussion_id IS 'Owning discussion ID'",
        "COMMENT ON COLUMN project_issue_comments.position IS 'Inline review anchor JSON'",
        "COMMENT ON COLUMN project_merge_request_comments.discussion_id IS 'Owning discussion ID'",
        "COMMENT ON COLUMN project_merge_request_comments.position IS 'Inline review anchor JSON'",
      ] {
        connection.execute_unprepared(statement).await?;
      }
    }
    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    // Threads and anchors are review metadata; dropping them is safe and keeps
    // the down path symmetric with `up`.
    for column in [
      ProjectIssueComments::DiscussionId,
      ProjectIssueComments::Position,
    ] {
      manager
        .alter_table(
          sea_query::Table::alter()
            .table(ProjectIssueComments::Table)
            .drop_column(column)
            .to_owned(),
        )
        .await?;
    }
    for column in [
      ProjectMergeRequestComments::DiscussionId,
      ProjectMergeRequestComments::Position,
    ] {
      manager
        .alter_table(
          sea_query::Table::alter()
            .table(ProjectMergeRequestComments::Table)
            .drop_column(column)
            .to_owned(),
        )
        .await?;
    }
    manager
      .drop_table(Table::drop().table(ProjectDiscussions::Table).to_owned())
      .await
  }
}
