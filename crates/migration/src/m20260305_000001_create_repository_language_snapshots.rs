use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(RepositoryLanguageSnapshots::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(RepositoryLanguageSnapshots::Id)
              .string()
              .not_null()
              .primary_key(),
          )
          .col(
            ColumnDef::new(RepositoryLanguageSnapshots::RepositoryId)
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(RepositoryLanguageSnapshots::BranchName)
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(RepositoryLanguageSnapshots::Revision)
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(RepositoryLanguageSnapshots::TotalBytes)
              .big_integer()
              .not_null(),
          )
          .col(
            ColumnDef::new(RepositoryLanguageSnapshots::AnalyzedAt)
              .timestamp_with_time_zone()
              .not_null(),
          )
          .col(
            ColumnDef::new(RepositoryLanguageSnapshots::CreatedAt)
              .timestamp_with_time_zone()
              .not_null(),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_repo_language_snapshots_repo_id")
              .from(
                RepositoryLanguageSnapshots::Table,
                RepositoryLanguageSnapshots::RepositoryId,
              )
              .to(Repositories::Table, Repositories::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_repo_language_snapshots_repo_branch_time")
          .table(RepositoryLanguageSnapshots::Table)
          .col(RepositoryLanguageSnapshots::RepositoryId)
          .col(RepositoryLanguageSnapshots::BranchName)
          .col(RepositoryLanguageSnapshots::AnalyzedAt)
          .if_not_exists()
          .to_owned(),
      )
      .await?;

    manager
      .create_table(
        Table::create()
          .table(RepositoryLanguageSnapshotItems::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(RepositoryLanguageSnapshotItems::Id)
              .string()
              .not_null()
              .primary_key(),
          )
          .col(
            ColumnDef::new(RepositoryLanguageSnapshotItems::SnapshotId)
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(RepositoryLanguageSnapshotItems::Language)
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(RepositoryLanguageSnapshotItems::Bytes)
              .big_integer()
              .not_null(),
          )
          .col(
            ColumnDef::new(RepositoryLanguageSnapshotItems::CreatedAt)
              .timestamp_with_time_zone()
              .not_null(),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_repo_language_snapshot_items_snapshot_id")
              .from(
                RepositoryLanguageSnapshotItems::Table,
                RepositoryLanguageSnapshotItems::SnapshotId,
              )
              .to(
                RepositoryLanguageSnapshots::Table,
                RepositoryLanguageSnapshots::Id,
              )
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_repo_language_snapshot_items_snapshot_id")
          .table(RepositoryLanguageSnapshotItems::Table)
          .col(RepositoryLanguageSnapshotItems::SnapshotId)
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
          .table(RepositoryLanguageSnapshotItems::Table)
          .to_owned(),
      )
      .await?;

    manager
      .drop_table(
        Table::drop()
          .table(RepositoryLanguageSnapshots::Table)
          .to_owned(),
      )
      .await
  }
}

#[derive(DeriveIden)]
enum RepositoryLanguageSnapshots {
  Table,
  Id,
  RepositoryId,
  BranchName,
  Revision,
  TotalBytes,
  AnalyzedAt,
  CreatedAt,
}

#[derive(DeriveIden)]
enum RepositoryLanguageSnapshotItems {
  Table,
  Id,
  SnapshotId,
  Language,
  Bytes,
  CreatedAt,
}

#[derive(DeriveIden)]
enum Repositories {
  Table,
  Id,
}
