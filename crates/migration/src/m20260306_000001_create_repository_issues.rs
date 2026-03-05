use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(RepositoryIssues::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(RepositoryIssues::Id)
              .string()
              .not_null()
              .primary_key(),
          )
          .col(
            ColumnDef::new(RepositoryIssues::RepositoryId)
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(RepositoryIssues::Number)
              .integer()
              .not_null(),
          )
          .col(ColumnDef::new(RepositoryIssues::Title).string().not_null())
          .col(ColumnDef::new(RepositoryIssues::Description).text().null())
          .col(ColumnDef::new(RepositoryIssues::Status).string().not_null())
          .col(
            ColumnDef::new(RepositoryIssues::AuthorUserId)
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(RepositoryIssues::AssigneeUserId)
              .string()
              .null(),
          )
          .col(
            ColumnDef::new(RepositoryIssues::CreatedAt)
              .timestamp_with_time_zone()
              .not_null(),
          )
          .col(
            ColumnDef::new(RepositoryIssues::UpdatedAt)
              .timestamp_with_time_zone()
              .not_null(),
          )
          .col(
            ColumnDef::new(RepositoryIssues::ClosedAt)
              .timestamp_with_time_zone()
              .null(),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_repository_issues_repository_id")
              .from(RepositoryIssues::Table, RepositoryIssues::RepositoryId)
              .to(Repositories::Table, Repositories::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_repository_issues_author_user_id")
              .from(RepositoryIssues::Table, RepositoryIssues::AuthorUserId)
              .to(Users::Table, Users::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_repository_issues_assignee_user_id")
              .from(RepositoryIssues::Table, RepositoryIssues::AssigneeUserId)
              .to(Users::Table, Users::Id)
              .on_delete(ForeignKeyAction::SetNull),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_repository_issues_repository_number")
          .table(RepositoryIssues::Table)
          .col(RepositoryIssues::RepositoryId)
          .col(RepositoryIssues::Number)
          .unique()
          .if_not_exists()
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_repository_issues_repository_status")
          .table(RepositoryIssues::Table)
          .col(RepositoryIssues::RepositoryId)
          .col(RepositoryIssues::Status)
          .if_not_exists()
          .to_owned(),
      )
      .await?;

    manager
      .create_table(
        Table::create()
          .table(RepositoryIssueComments::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(RepositoryIssueComments::Id)
              .string()
              .not_null()
              .primary_key(),
          )
          .col(
            ColumnDef::new(RepositoryIssueComments::IssueId)
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(RepositoryIssueComments::AuthorUserId)
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(RepositoryIssueComments::Content)
              .text()
              .not_null(),
          )
          .col(
            ColumnDef::new(RepositoryIssueComments::CreatedAt)
              .timestamp_with_time_zone()
              .not_null(),
          )
          .col(
            ColumnDef::new(RepositoryIssueComments::UpdatedAt)
              .timestamp_with_time_zone()
              .not_null(),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_repository_issue_comments_issue_id")
              .from(
                RepositoryIssueComments::Table,
                RepositoryIssueComments::IssueId,
              )
              .to(RepositoryIssues::Table, RepositoryIssues::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_repository_issue_comments_author_user_id")
              .from(
                RepositoryIssueComments::Table,
                RepositoryIssueComments::AuthorUserId,
              )
              .to(Users::Table, Users::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_repository_issue_comments_issue_created_at")
          .table(RepositoryIssueComments::Table)
          .col(RepositoryIssueComments::IssueId)
          .col(RepositoryIssueComments::CreatedAt)
          .if_not_exists()
          .to_owned(),
      )
      .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(
        Table::drop()
          .table(RepositoryIssueComments::Table)
          .to_owned(),
      )
      .await?;

    manager
      .drop_table(Table::drop().table(RepositoryIssues::Table).to_owned())
      .await
  }
}

#[derive(DeriveIden)]
enum RepositoryIssues {
  Table,
  Id,
  RepositoryId,
  Number,
  Title,
  Description,
  Status,
  AuthorUserId,
  AssigneeUserId,
  CreatedAt,
  UpdatedAt,
  ClosedAt,
}

#[derive(DeriveIden)]
enum RepositoryIssueComments {
  Table,
  Id,
  IssueId,
  AuthorUserId,
  Content,
  CreatedAt,
  UpdatedAt,
}

#[derive(DeriveIden)]
enum Repositories {
  Table,
  Id,
}

#[derive(DeriveIden)]
enum Users {
  Table,
  Id,
}
