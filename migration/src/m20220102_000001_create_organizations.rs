use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Organizations::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(Organizations::Id)
              .string()
              .not_null()
              .primary_key(),
          )
          .col(ColumnDef::new(Organizations::Key).string().not_null())
          .col(ColumnDef::new(Organizations::Name).string().not_null())
          .col(ColumnDef::new(Organizations::Status).string().not_null())
          .col(
            ColumnDef::new(Organizations::CreatedAt)
              .timestamp_with_time_zone()
              .not_null(),
          )
          .col(
            ColumnDef::new(Organizations::UpdatedAt)
              .timestamp_with_time_zone()
              .not_null(),
          )
          .col(
            ColumnDef::new(Organizations::DeletedAt)
              .timestamp_with_time_zone()
              .null(),
          )
          .index(
            Index::create()
              .name("idx_organizations_key")
              .table(Organizations::Table)
              .col(Organizations::Key)
              .unique(),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_table(
        Table::create()
          .table(OrganizationMembers::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(OrganizationMembers::Id)
              .string()
              .not_null()
              .primary_key(),
          )
          .col(
            ColumnDef::new(OrganizationMembers::OrganizationId)
              .string()
              .not_null(),
          )
          .col(ColumnDef::new(OrganizationMembers::UserId).string().not_null())
          .col(ColumnDef::new(OrganizationMembers::Role).string().not_null())
          .col(
            ColumnDef::new(OrganizationMembers::CreatedAt)
              .timestamp_with_time_zone()
              .not_null(),
          )
          .col(
            ColumnDef::new(OrganizationMembers::UpdatedAt)
              .timestamp_with_time_zone()
              .not_null(),
          )
          .col(
            ColumnDef::new(OrganizationMembers::DeletedAt)
              .timestamp_with_time_zone()
              .null(),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_organization_members_org_id")
              .from(
                OrganizationMembers::Table,
                OrganizationMembers::OrganizationId,
              )
              .to(Organizations::Table, Organizations::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_organization_members_user_id")
              .from(OrganizationMembers::Table, OrganizationMembers::UserId)
              .to(Users::Table, Users::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .index(
            Index::create()
              .name("idx_organization_members_org_user")
              .table(OrganizationMembers::Table)
              .col(OrganizationMembers::OrganizationId)
              .col(OrganizationMembers::UserId)
              .unique(),
          )
          .index(
            Index::create()
              .name("idx_organization_members_user_id")
              .table(OrganizationMembers::Table)
              .col(OrganizationMembers::UserId),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(OrganizationMembers::Table).to_owned())
      .await?;

    manager
      .drop_table(Table::drop().table(Organizations::Table).to_owned())
      .await
  }
}

#[derive(DeriveIden)]
enum Organizations {
  Table,
  Id,
  Key,
  Name,
  Status,
  CreatedAt,
  UpdatedAt,
  DeletedAt,
}

#[derive(DeriveIden)]
enum OrganizationMembers {
  Table,
  Id,
  OrganizationId,
  UserId,
  Role,
  CreatedAt,
  UpdatedAt,
  DeletedAt,
}

#[derive(DeriveIden)]
enum Users {
  Table,
  Id,
}

