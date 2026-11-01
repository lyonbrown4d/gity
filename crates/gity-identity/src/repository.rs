use gity_entities::{user_passwords, users};
use sea_orm::{
  ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};

use crate::UserIdentity;
mod accounts;
mod authorization;
mod credentials;

#[derive(Clone)]
pub struct IdentityRepository {
  connection: DatabaseConnection,
}

impl IdentityRepository {
  pub fn new(connection: DatabaseConnection) -> Self {
    Self { connection }
  }

  pub async fn user_by_id(&self, id: i64) -> Result<Option<UserIdentity>, DbErr> {
    users::Entity::find_by_id(id)
      .one(&self.connection)
      .await
      .map(|v| v.map(user_identity))
  }

  pub async fn user_by_username(&self, username: &str) -> Result<Option<UserIdentity>, DbErr> {
    users::Entity::find()
      .filter(users::Column::Username.eq(username))
      .one(&self.connection)
      .await
      .map(|v| v.map(user_identity))
  }

  pub async fn user_model_by_username(
    &self,
    username: &str,
  ) -> Result<Option<users::Model>, DbErr> {
    users::Entity::find()
      .filter(users::Column::Username.eq(username))
      .one(&self.connection)
      .await
  }
  pub async fn password_hash(&self, user_id: i64) -> Result<Option<String>, DbErr> {
    user_passwords::Entity::find_by_id(user_id)
      .one(&self.connection)
      .await
      .map(|v| v.map(|row| row.password_hash))
  }
  pub async fn users_page(
    &self,
    page: u64,
    page_size: u64,
  ) -> Result<(Vec<users::Model>, u64), DbErr> {
    let paginator = users::Entity::find()
      .order_by_asc(users::Column::Id)
      .paginate(&self.connection, page_size);
    let total = paginator.num_items().await?;
    let rows = paginator.fetch_page(page.saturating_sub(1)).await?;
    Ok((rows, total))
  }

  pub async fn user_model(&self, id: i64) -> Result<Option<users::Model>, DbErr> {
    users::Entity::find_by_id(id).one(&self.connection).await
  }
}

pub(super) fn user_identity(user: users::Model) -> UserIdentity {
  UserIdentity {
    id: user.id,
    username: user.username,
    status: user.status,
    is_super_admin: user.is_super_admin == 1,
  }
}
