use crate::{NewProjectToken, RepositoryAccessRepository};
use gity_core::AppError;
use gity_entities::{project_access_tokens, project_deploy_keys, user_ssh_keys};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait,
};

impl RepositoryAccessRepository {
  pub(crate) async fn list_tokens(
    &self,
    project_id: i64,
    kind: &str,
  ) -> Result<Vec<project_access_tokens::Model>, AppError> {
    Ok(
      project_access_tokens::Entity::find()
        .filter(project_access_tokens::Column::ProjectId.eq(project_id))
        .filter(project_access_tokens::Column::Kind.eq(kind))
        .order_by_desc(project_access_tokens::Column::Id)
        .all(&self.connection)
        .await?,
    )
  }

  pub(crate) async fn token_by_hash(
    &self,
    digest: &str,
  ) -> Result<Option<project_access_tokens::Model>, AppError> {
    Ok(
      project_access_tokens::Entity::find()
        .filter(project_access_tokens::Column::Token.eq(digest))
        .one(&self.connection)
        .await?,
    )
  }

  pub(crate) async fn scoped_token(
    &self,
    project_id: i64,
    id: i64,
  ) -> Result<Option<project_access_tokens::Model>, AppError> {
    Ok(
      project_access_tokens::Entity::find()
        .filter(project_access_tokens::Column::ProjectId.eq(project_id))
        .filter(project_access_tokens::Column::Id.eq(id))
        .one(&self.connection)
        .await?,
    )
  }

  pub(crate) async fn create_token(
    &self,
    input: NewProjectToken,
  ) -> Result<project_access_tokens::Model, AppError> {
    let now = chrono::Utc::now().naive_utc();
    Ok(
      project_access_tokens::ActiveModel {
        id: Set(gity_core::new_id()),
        project_id: Set(input.project_id),
        kind: Set(input.kind),
        name: Set(input.name),
        username: Set(input.username),
        token: Set(input.token_digest),
        scopes: Set(input.scopes),
        created_by_user_id: Set(input.created_by_user_id),
        expires_at: Set(input.expires_at),
        revoked_at: Set(None),
        last_used_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
      }
      .insert(&self.connection)
      .await?,
    )
  }

  pub(crate) async fn mark_token_used(
    &self,
    token: project_access_tokens::Model,
  ) -> Result<(), AppError> {
    let now = chrono::Utc::now().naive_utc();
    project_access_tokens::ActiveModel {
      id: Set(token.id),
      last_used_at: Set(Some(now)),
      updated_at: Set(now),
      ..Default::default()
    }
    .update(&self.connection)
    .await?;
    Ok(())
  }

  pub(crate) async fn revoke_token(
    &self,
    token: project_access_tokens::Model,
  ) -> Result<(), AppError> {
    let now = chrono::Utc::now().naive_utc();
    project_access_tokens::ActiveModel {
      id: Set(token.id),
      revoked_at: Set(Some(now)),
      updated_at: Set(now),
      ..Default::default()
    }
    .update(&self.connection)
    .await?;
    Ok(())
  }

  pub(crate) async fn list_deploy_keys(
    &self,
    project_id: i64,
  ) -> Result<Vec<project_deploy_keys::Model>, AppError> {
    Ok(
      project_deploy_keys::Entity::find()
        .filter(project_deploy_keys::Column::ProjectId.eq(project_id))
        .order_by_desc(project_deploy_keys::Column::Id)
        .all(&self.connection)
        .await?,
    )
  }

  pub(crate) async fn scoped_deploy_key(
    &self,
    project_id: i64,
    id: i64,
  ) -> Result<Option<project_deploy_keys::Model>, AppError> {
    Ok(
      project_deploy_keys::Entity::find()
        .filter(project_deploy_keys::Column::ProjectId.eq(project_id))
        .filter(project_deploy_keys::Column::Id.eq(id))
        .one(&self.connection)
        .await?,
    )
  }

  pub(crate) async fn create_deploy_key(
    &self,
    project_id: i64,
    title: String,
    public_key: String,
    fingerprint: String,
    can_push: bool,
    created_by_user_id: i64,
  ) -> Result<project_deploy_keys::Model, AppError> {
    let tx = self.connection.begin().await?;
    if user_ssh_keys::Entity::find()
      .filter(user_ssh_keys::Column::Fingerprint.eq(&fingerprint))
      .one(&tx)
      .await?
      .is_some()
    {
      return Err(AppError::conflict(
        "SSH fingerprint already registered as a personal key",
      ));
    }
    let now = chrono::Utc::now().naive_utc();
    let model = project_deploy_keys::ActiveModel {
      id: Set(gity_core::new_id()),
      project_id: Set(project_id),
      title: Set(title),
      fingerprint: Set(fingerprint),
      public_key: Set(public_key),
      can_push: Set(i32::from(can_push)),
      created_by_user_id: Set(created_by_user_id),
      last_used_at: Set(None),
      created_at: Set(now),
      updated_at: Set(now),
    }
    .insert(&tx)
    .await?;
    gity_persistence::audit_deploy_key(&tx, &model, "insert", Some(created_by_user_id)).await?;
    tx.commit().await?;
    Ok(model)
  }

  pub(crate) async fn delete_deploy_key(
    &self,
    key: project_deploy_keys::Model,
  ) -> Result<(), AppError> {
    let tx = self.connection.begin().await?;
    let current = project_deploy_keys::Entity::find_by_id(key.id)
      .filter(project_deploy_keys::Column::ProjectId.eq(key.project_id))
      .one(&tx)
      .await?;
    if let Some(current) = current {
      project_deploy_keys::Entity::delete_by_id(key.id)
        .exec(&tx)
        .await?;
      gity_persistence::audit_deploy_key(&tx, &current, "delete", None).await?;
    }
    tx.commit().await?;
    Ok(())
  }
}
