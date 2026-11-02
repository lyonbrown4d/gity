use chrono::NaiveDateTime;
use gity_entities::{bootstrap_claim, organization_members, organizations, user_passwords, users};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, DatabaseTransaction, DbErr, EntityTrait, QueryFilter, Set,
  TransactionTrait,
};

use super::IdentityRepository;

pub struct BootstrapAccountCommand {
  pub user_id: i64,
  pub organization_id: i64,
  pub membership_id: i64,
  pub username: String,
  pub email: Option<String>,
  pub password_hash: String,
  pub timestamp: NaiveDateTime,
}

pub struct BootstrapAccount {
  pub user: users::Model,
  pub organization_id: i64,
}

pub struct BootstrapGuard {
  transaction: Option<DatabaseTransaction>,
}

impl IdentityRepository {
  pub async fn bootstrap_claimed(&self) -> Result<bool, DbErr> {
    bootstrap_claim::Entity::find_by_id(1)
      .one(&self.connection)
      .await
      .map(|claim| claim.is_some())
  }

  /// Inserts the unique bootstrap claim before any password file is accessed.
  pub async fn begin_bootstrap(&self) -> Result<Option<BootstrapGuard>, DbErr> {
    let transaction = self.connection.begin().await?;
    if let Err(error) = (bootstrap_claim::ActiveModel { id: Set(1) })
      .insert(&transaction)
      .await
    {
      transaction.rollback().await?;
      if self.bootstrap_claimed().await? {
        return Ok(None);
      }
      return Err(error);
    }
    Ok(Some(BootstrapGuard {
      transaction: Some(transaction),
    }))
  }
}

impl BootstrapGuard {
  pub async fn create_account(
    &mut self,
    command: BootstrapAccountCommand,
  ) -> Result<BootstrapAccount, DbErr> {
    let transaction = self
      .transaction
      .as_ref()
      .ok_or_else(|| DbErr::Custom("bootstrap transaction is closed".to_owned()))?;
    if users::Entity::find()
      .filter(users::Column::Username.eq(&command.username))
      .one(transaction)
      .await?
      .is_some()
    {
      return Err(DbErr::Custom(
        "bootstrap username already exists".to_owned(),
      ));
    }
    let user = users::ActiveModel {
      id: Set(command.user_id),
      username: Set(command.username.clone()),
      display_name: Set(command.username.clone()),
      email: Set(command.email),
      created_at: Set(command.timestamp),
      updated_at: Set(command.timestamp),
      is_super_admin: Set(1),
      status: Set("active".to_owned()),
    }
    .insert(transaction)
    .await?;
    user_passwords::ActiveModel {
      user_id: Set(command.user_id),
      password_hash: Set(command.password_hash),
    }
    .insert(transaction)
    .await?;
    let path = format!(
      "{}-{}",
      command.username.to_lowercase(),
      command.organization_id
    );
    organizations::ActiveModel {
      id: Set(command.organization_id),
      name: Set(command.username),
      path_key: Set(path.clone()),
      full_path: Set(path),
      description: Set(Some("Personal namespace".to_owned())),
      visibility: Set("private".to_owned()),
      created_at: Set(command.timestamp),
      updated_at: Set(command.timestamp),
    }
    .insert(transaction)
    .await?;
    organization_members::ActiveModel {
      id: Set(command.membership_id),
      organization_id: Set(command.organization_id),
      user_id: Set(command.user_id),
      role: Set("owner".to_owned()),
      created_at: Set(command.timestamp),
      updated_at: Set(command.timestamp),
    }
    .insert(transaction)
    .await?;
    Ok(BootstrapAccount {
      user,
      organization_id: command.organization_id,
    })
  }

  pub async fn commit(&mut self) -> Result<(), DbErr> {
    if let Some(transaction) = self.transaction.take() {
      transaction.commit().await
    } else {
      Err(DbErr::Custom("bootstrap transaction is closed".to_owned()))
    }
  }

  pub async fn rollback(&mut self) -> Result<(), DbErr> {
    if let Some(transaction) = self.transaction.take() {
      transaction.rollback().await
    } else {
      Ok(())
    }
  }
}
