use super::{IdentityRepository, user_lock::lock_user};
use gity_entities::{project_deploy_keys, user_ssh_keys};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait,
};

impl IdentityRepository {
  pub async fn personal_ssh_keys(&self, user_id: i64) -> Result<Vec<user_ssh_keys::Model>, DbErr> {
    user_ssh_keys::Entity::find()
      .filter(user_ssh_keys::Column::UserId.eq(user_id))
      .order_by_desc(user_ssh_keys::Column::Id)
      .all(&self.connection)
      .await
  }

  pub async fn personal_ssh_key(
    &self,
    fingerprint: &str,
  ) -> Result<Option<user_ssh_keys::Model>, DbErr> {
    user_ssh_keys::Entity::find()
      .filter(user_ssh_keys::Column::Fingerprint.eq(fingerprint))
      .one(&self.connection)
      .await
  }

  /// Includes inactive creators/projects: a fingerprint cannot have two identity modes.
  pub async fn deployment_fingerprint_exists(&self, fingerprint: &str) -> Result<bool, DbErr> {
    Ok(
      project_deploy_keys::Entity::find()
        .filter(project_deploy_keys::Column::Fingerprint.eq(fingerprint))
        .one(&self.connection)
        .await?
        .is_some(),
    )
  }

  pub async fn add_personal_ssh_key(
    &self,
    command: crate::PersonalSshKeyCommand,
  ) -> Result<user_ssh_keys::Model, DbErr> {
    let tx = self.connection.begin().await?;
    if lock_user(&tx, command.user_id)
      .await?
      .is_none_or(|user| user.status != "active")
    {
      return Err(DbErr::RecordNotFound("SSH key owner unavailable".into()));
    }
    if project_deploy_keys::Entity::find()
      .filter(project_deploy_keys::Column::Fingerprint.eq(&command.fingerprint))
      .one(&tx)
      .await?
      .is_some()
    {
      return Err(DbErr::Custom(
        "SSH fingerprint already registered as a deploy key".into(),
      ));
    }
    let now = chrono::Utc::now().naive_utc();
    let key = user_ssh_keys::ActiveModel {
      id: Set(command.id),
      user_id: Set(command.user_id),
      title: Set(command.title),
      public_key: Set(command.public_key),
      fingerprint: Set(command.fingerprint),
      last_used_at: Set(None),
      created_at: Set(now),
      updated_at: Set(now),
    }
    .insert(&tx)
    .await?;
    tx.commit().await?;
    Ok(key)
  }

  pub async fn delete_personal_ssh_key(&self, user_id: i64, id: i64) -> Result<bool, DbErr> {
    Ok(
      user_ssh_keys::Entity::delete_many()
        .filter(user_ssh_keys::Column::UserId.eq(user_id))
        .filter(user_ssh_keys::Column::Id.eq(id))
        .exec(&self.connection)
        .await?
        .rows_affected
        > 0,
    )
  }

  /// Conditional update prevents a revoked key from being recreated or used by a new command.
  pub async fn touch_personal_ssh_key(&self, user_id: i64, id: i64) -> Result<bool, DbErr> {
    Ok(
      user_ssh_keys::Entity::update_many()
        .col_expr(
          user_ssh_keys::Column::LastUsedAt,
          sea_orm::sea_query::Expr::value(chrono::Utc::now().naive_utc()),
        )
        .filter(user_ssh_keys::Column::UserId.eq(user_id))
        .filter(user_ssh_keys::Column::Id.eq(id))
        .exec(&self.connection)
        .await?
        .rows_affected
        > 0,
    )
  }
}
