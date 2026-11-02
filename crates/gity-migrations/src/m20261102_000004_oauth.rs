use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum OauthClients {
  Table,
  Id,
  ClientId,
  Name,
  RedirectUris,
  CreatedBy,
  IsDynamic,
  CreatedAt,
}
#[derive(DeriveIden)]
enum OauthPendingRequests {
  Table,
  Id,
  RequestKey,
  ClientId,
  UserId,
  RedirectUri,
  CodeChallenge,
  Scopes,
  State,
  Resource,
  Nonce,
  AdminMode,
  Status,
  ExpiresAt,
}
#[derive(DeriveIden)]
enum OauthAuthorizationCodes {
  Table,
  Id,
  CodeHash,
  ClientId,
  UserId,
  RedirectUri,
  CodeChallenge,
  Scopes,
  Resource,
  AdminMode,
  ExpiresAt,
  RedeemedAt,
}
#[derive(DeriveIden)]
enum OauthTokens {
  Table,
  Id,
  TokenHash,
  FamilyKey,
  ClientId,
  UserId,
  Kind,
  Scopes,
  Resource,
  AdminMode,
  ExpiresAt,
  RevokedAt,
  ReplacedByHash,
}
#[derive(DeriveIden)]
enum OauthTokenFamilies {
  Table,
  Id,
  FamilyKey,
  CreatedAt,
  RevokedAt,
}

fn id_column(name: impl IntoIden) -> ColumnDef {
  ColumnDef::new(name)
    .big_integer()
    .not_null()
    .primary_key()
    .to_owned()
}
fn text(name: impl IntoIden) -> ColumnDef {
  ColumnDef::new(name).string().not_null().to_owned()
}
fn nullable_text(name: impl IntoIden) -> ColumnDef {
  ColumnDef::new(name).string().null().to_owned()
}
fn timestamp(name: impl IntoIden) -> ColumnDef {
  ColumnDef::new(name).date_time().not_null().to_owned()
}
fn nullable_timestamp(name: impl IntoIden) -> ColumnDef {
  ColumnDef::new(name).date_time().null().to_owned()
}
fn integer(name: impl IntoIden) -> ColumnDef {
  ColumnDef::new(name).big_integer().not_null().to_owned()
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let mut clients = Table::create();
    clients
      .table(OauthClients::Table)
      .col(id_column(OauthClients::Id))
      .col(text(OauthClients::ClientId))
      .col(text(OauthClients::Name))
      .col(ColumnDef::new(OauthClients::RedirectUris).json().not_null())
      .col(ColumnDef::new(OauthClients::CreatedBy).big_integer().null())
      .col(ColumnDef::new(OauthClients::IsDynamic).boolean().not_null())
      .col(timestamp(OauthClients::CreatedAt));
    manager.create_table(clients.to_owned()).await?;
    manager
      .create_index(
        Index::create()
          .name("uq_oauth_client_id")
          .table(OauthClients::Table)
          .col(OauthClients::ClientId)
          .unique()
          .to_owned(),
      )
      .await?;

    let mut pending = Table::create();
    pending
      .table(OauthPendingRequests::Table)
      .col(id_column(OauthPendingRequests::Id))
      .col(text(OauthPendingRequests::RequestKey))
      .col(text(OauthPendingRequests::ClientId))
      .col(
        ColumnDef::new(OauthPendingRequests::UserId)
          .big_integer()
          .null(),
      )
      .col(text(OauthPendingRequests::RedirectUri))
      .col(text(OauthPendingRequests::CodeChallenge))
      .col(text(OauthPendingRequests::Scopes))
      .col(text(OauthPendingRequests::State))
      .col(text(OauthPendingRequests::Resource))
      .col(nullable_text(OauthPendingRequests::Nonce))
      .col(
        ColumnDef::new(OauthPendingRequests::AdminMode)
          .boolean()
          .not_null(),
      )
      .col(text(OauthPendingRequests::Status))
      .col(timestamp(OauthPendingRequests::ExpiresAt));
    manager.create_table(pending.to_owned()).await?;
    manager
      .create_index(
        Index::create()
          .name("uq_oauth_pending_key")
          .table(OauthPendingRequests::Table)
          .col(OauthPendingRequests::RequestKey)
          .unique()
          .to_owned(),
      )
      .await?;
    manager
      .create_index(
        Index::create()
          .name("idx_oauth_pending_expiry")
          .table(OauthPendingRequests::Table)
          .col(OauthPendingRequests::ExpiresAt)
          .to_owned(),
      )
      .await?;

    let mut codes = Table::create();
    codes
      .table(OauthAuthorizationCodes::Table)
      .col(id_column(OauthAuthorizationCodes::Id))
      .col(text(OauthAuthorizationCodes::CodeHash))
      .col(text(OauthAuthorizationCodes::ClientId))
      .col(integer(OauthAuthorizationCodes::UserId))
      .col(text(OauthAuthorizationCodes::RedirectUri))
      .col(text(OauthAuthorizationCodes::CodeChallenge))
      .col(text(OauthAuthorizationCodes::Scopes))
      .col(text(OauthAuthorizationCodes::Resource))
      .col(
        ColumnDef::new(OauthAuthorizationCodes::AdminMode)
          .boolean()
          .not_null(),
      )
      .col(timestamp(OauthAuthorizationCodes::ExpiresAt))
      .col(nullable_timestamp(OauthAuthorizationCodes::RedeemedAt));
    manager.create_table(codes.to_owned()).await?;
    manager
      .create_index(
        Index::create()
          .name("uq_oauth_code_hash")
          .table(OauthAuthorizationCodes::Table)
          .col(OauthAuthorizationCodes::CodeHash)
          .unique()
          .to_owned(),
      )
      .await?;

    let mut tokens = Table::create();
    tokens
      .table(OauthTokens::Table)
      .col(id_column(OauthTokens::Id))
      .col(text(OauthTokens::TokenHash))
      .col(text(OauthTokens::FamilyKey))
      .col(text(OauthTokens::ClientId))
      .col(integer(OauthTokens::UserId))
      .col(text(OauthTokens::Kind))
      .col(text(OauthTokens::Scopes))
      .col(text(OauthTokens::Resource))
      .col(ColumnDef::new(OauthTokens::AdminMode).boolean().not_null())
      .col(timestamp(OauthTokens::ExpiresAt))
      .col(nullable_timestamp(OauthTokens::RevokedAt))
      .col(nullable_text(OauthTokens::ReplacedByHash));
    manager.create_table(tokens.to_owned()).await?;
    manager
      .create_index(
        Index::create()
          .name("uq_oauth_token_hash")
          .table(OauthTokens::Table)
          .col(OauthTokens::TokenHash)
          .unique()
          .to_owned(),
      )
      .await?;
    manager
      .create_index(
        Index::create()
          .name("idx_oauth_token_family")
          .table(OauthTokens::Table)
          .col(OauthTokens::FamilyKey)
          .to_owned(),
      )
      .await?;

    let mut families = Table::create();
    families
      .table(OauthTokenFamilies::Table)
      .comment("Durable OAuth token family revocation markers")
      .col(id_column(OauthTokenFamilies::Id))
      .col(text(OauthTokenFamilies::FamilyKey))
      .col(timestamp(OauthTokenFamilies::CreatedAt))
      .col(nullable_timestamp(OauthTokenFamilies::RevokedAt));
    manager.create_table(families.to_owned()).await?;
    manager
      .create_index(
        Index::create()
          .name("uq_oauth_token_family_key")
          .table(OauthTokenFamilies::Table)
          .col(OauthTokenFamilies::FamilyKey)
          .unique()
          .to_owned(),
      )
      .await?;
    Ok(())
  }

  async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
    Err(DbErr::Migration(
      "OAuth credentials cannot be discarded; restore a backup".into(),
    ))
  }
}
