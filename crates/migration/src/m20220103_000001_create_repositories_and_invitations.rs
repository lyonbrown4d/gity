use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Repositories::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(Repositories::Id)
              .string()
              .not_null()
              .primary_key(),
          )
          .col(
            ColumnDef::new(Repositories::OrganizationId)
              .string()
              .not_null(),
          )
          .col(ColumnDef::new(Repositories::Key).string().not_null())
          .col(ColumnDef::new(Repositories::Name).string().not_null())
          .col(ColumnDef::new(Repositories::Description).text().null())
          .col(ColumnDef::new(Repositories::Visibility).string().not_null())
          .col(
            ColumnDef::new(Repositories::DefaultBranch)
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(Repositories::CreatedByUserId)
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(Repositories::CreatedAt)
              .timestamp_with_time_zone()
              .not_null(),
          )
          .col(
            ColumnDef::new(Repositories::UpdatedAt)
              .timestamp_with_time_zone()
              .not_null(),
          )
          .col(
            ColumnDef::new(Repositories::DeletedAt)
              .timestamp_with_time_zone()
              .null(),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_repositories_org_id")
              .from(Repositories::Table, Repositories::OrganizationId)
              .to(Organizations::Table, Organizations::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_repositories_created_by")
              .from(Repositories::Table, Repositories::CreatedByUserId)
              .to(Users::Table, Users::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_repositories_org_key")
          .table(Repositories::Table)
          .col(Repositories::OrganizationId)
          .col(Repositories::Key)
          .unique()
          .if_not_exists()
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_repositories_org_id")
          .table(Repositories::Table)
          .col(Repositories::OrganizationId)
          .if_not_exists()
          .to_owned(),
      )
      .await?;

    manager
      .create_table(
        Table::create()
          .table(OrganizationInvitations::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(OrganizationInvitations::Id)
              .string()
              .not_null()
              .primary_key(),
          )
          .col(
            ColumnDef::new(OrganizationInvitations::OrganizationId)
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(OrganizationInvitations::Email)
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(OrganizationInvitations::Role)
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(OrganizationInvitations::Status)
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(OrganizationInvitations::InvitedByUserId)
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(OrganizationInvitations::AcceptedByUserId)
              .string()
              .null(),
          )
          .col(
            ColumnDef::new(OrganizationInvitations::ExpiresAt)
              .timestamp_with_time_zone()
              .null(),
          )
          .col(
            ColumnDef::new(OrganizationInvitations::CreatedAt)
              .timestamp_with_time_zone()
              .not_null(),
          )
          .col(
            ColumnDef::new(OrganizationInvitations::UpdatedAt)
              .timestamp_with_time_zone()
              .not_null(),
          )
          .col(
            ColumnDef::new(OrganizationInvitations::DeletedAt)
              .timestamp_with_time_zone()
              .null(),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_org_invitations_org_id")
              .from(
                OrganizationInvitations::Table,
                OrganizationInvitations::OrganizationId,
              )
              .to(Organizations::Table, Organizations::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_org_invitations_invited_by")
              .from(
                OrganizationInvitations::Table,
                OrganizationInvitations::InvitedByUserId,
              )
              .to(Users::Table, Users::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_org_invitations_accepted_by")
              .from(
                OrganizationInvitations::Table,
                OrganizationInvitations::AcceptedByUserId,
              )
              .to(Users::Table, Users::Id)
              .on_delete(ForeignKeyAction::SetNull),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_org_invitations_org_email_status")
          .table(OrganizationInvitations::Table)
          .col(OrganizationInvitations::OrganizationId)
          .col(OrganizationInvitations::Email)
          .col(OrganizationInvitations::Status)
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
          .table(OrganizationInvitations::Table)
          .to_owned(),
      )
      .await?;

    manager
      .drop_table(Table::drop().table(Repositories::Table).to_owned())
      .await
  }
}

#[derive(DeriveIden)]
enum Repositories {
  Table,
  Id,
  OrganizationId,
  Key,
  Name,
  Description,
  Visibility,
  DefaultBranch,
  CreatedByUserId,
  CreatedAt,
  UpdatedAt,
  DeletedAt,
}

#[derive(DeriveIden)]
enum OrganizationInvitations {
  Table,
  Id,
  OrganizationId,
  Email,
  Role,
  Status,
  InvitedByUserId,
  AcceptedByUserId,
  ExpiresAt,
  CreatedAt,
  UpdatedAt,
  DeletedAt,
}

#[derive(DeriveIden)]
enum Organizations {
  Table,
  Id,
}

#[derive(DeriveIden)]
enum Users {
  Table,
  Id,
}
