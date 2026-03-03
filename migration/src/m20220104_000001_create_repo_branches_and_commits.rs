use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(RepositoryBranches::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(RepositoryBranches::Id)
              .string()
              .not_null()
              .primary_key(),
          )
          .col(
            ColumnDef::new(RepositoryBranches::RepositoryId)
              .string()
              .not_null(),
          )
          .col(ColumnDef::new(RepositoryBranches::Name).string().not_null())
          .col(
            ColumnDef::new(RepositoryBranches::IsProtected)
              .boolean()
              .not_null()
              .default(false),
          )
          .col(ColumnDef::new(RepositoryBranches::LastCommitSha).string().null())
          .col(
            ColumnDef::new(RepositoryBranches::CreatedAt)
              .timestamp_with_time_zone()
              .not_null(),
          )
          .col(
            ColumnDef::new(RepositoryBranches::UpdatedAt)
              .timestamp_with_time_zone()
              .not_null(),
          )
          .col(
            ColumnDef::new(RepositoryBranches::DeletedAt)
              .timestamp_with_time_zone()
              .null(),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_repo_branches_repo_id")
              .from(RepositoryBranches::Table, RepositoryBranches::RepositoryId)
              .to(Repositories::Table, Repositories::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .index(
            Index::create()
              .name("idx_repo_branches_repo_name")
              .table(RepositoryBranches::Table)
              .col(RepositoryBranches::RepositoryId)
              .col(RepositoryBranches::Name)
              .unique(),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_table(
        Table::create()
          .table(RepositoryCommits::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(RepositoryCommits::Id)
              .string()
              .not_null()
              .primary_key(),
          )
          .col(
            ColumnDef::new(RepositoryCommits::RepositoryId)
              .string()
              .not_null(),
          )
          .col(ColumnDef::new(RepositoryCommits::BranchName).string().not_null())
          .col(ColumnDef::new(RepositoryCommits::CommitSha).string().not_null())
          .col(ColumnDef::new(RepositoryCommits::Message).string().not_null())
          .col(
            ColumnDef::new(RepositoryCommits::AuthorUserId)
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(RepositoryCommits::CreatedAt)
              .timestamp_with_time_zone()
              .not_null(),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_repo_commits_repo_id")
              .from(RepositoryCommits::Table, RepositoryCommits::RepositoryId)
              .to(Repositories::Table, Repositories::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_repo_commits_author")
              .from(RepositoryCommits::Table, RepositoryCommits::AuthorUserId)
              .to(Users::Table, Users::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .index(
            Index::create()
              .name("idx_repo_commits_repo_sha")
              .table(RepositoryCommits::Table)
              .col(RepositoryCommits::RepositoryId)
              .col(RepositoryCommits::CommitSha)
              .unique(),
          )
          .index(
            Index::create()
              .name("idx_repo_commits_repo_branch")
              .table(RepositoryCommits::Table)
              .col(RepositoryCommits::RepositoryId)
              .col(RepositoryCommits::BranchName),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(RepositoryCommits::Table).to_owned())
      .await?;

    manager
      .drop_table(Table::drop().table(RepositoryBranches::Table).to_owned())
      .await
  }
}

#[derive(DeriveIden)]
enum RepositoryBranches {
  Table,
  Id,
  RepositoryId,
  Name,
  IsProtected,
  LastCommitSha,
  CreatedAt,
  UpdatedAt,
  DeletedAt,
}

#[derive(DeriveIden)]
enum RepositoryCommits {
  Table,
  Id,
  RepositoryId,
  BranchName,
  CommitSha,
  Message,
  AuthorUserId,
  CreatedAt,
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

