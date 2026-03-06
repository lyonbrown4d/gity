use crate::BaseRepository;
use chrono::Utc;
use entity::users;
use sea_orm::{
  ColumnTrait, Condition, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, IntoActiveModel,
  PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set, prelude::DateTimeWithTimeZone,
};

#[derive(Clone)]
pub struct UsersRepository<C: ConnectionTrait = DatabaseConnection> {
  base: BaseRepository<C>,
}

impl<C: ConnectionTrait> UsersRepository<C> {
  pub fn new(conn: C) -> Self {
    Self {
      base: BaseRepository::new(conn),
    }
  }

  pub fn connection(&self) -> &C {
    self.base.connection()
  }

  pub async fn find_user_by_username_or_email(
    &self,
    identity: &str,
  ) -> Result<Option<users::Model>, DbErr> {
    users::Entity::find()
      .filter(
        Condition::any()
          .add(users::Column::Username.eq(identity.to_string()))
          .add(users::Column::Email.eq(identity.to_string())),
      )
      .one(self.connection())
      .await
  }

  pub async fn find_duplicate_user_by_username_or_email(
    &self,
    username: &str,
    email: &str,
  ) -> Result<Option<users::Model>, DbErr> {
    users::Entity::find()
      .filter(
        Condition::any()
          .add(users::Column::Username.eq(username.to_string()))
          .add(users::Column::Email.eq(email.to_string())),
      )
      .one(self.connection())
      .await
  }

  pub async fn find_active_user_by_id(&self, user_id: &str) -> Result<Option<users::Model>, DbErr> {
    users::Entity::find_by_id(user_id.to_string())
      .filter(users::Column::DeletedAt.is_null())
      .one(self.connection())
      .await
  }

  pub async fn list_active_users(&self, limit: Option<u64>) -> Result<Vec<users::Model>, DbErr> {
    let mut query = users::Entity::find()
      .filter(users::Column::DeletedAt.is_null())
      .order_by_desc(users::Column::CreatedAt);

    if let Some(limit) = limit {
      query = query.limit(limit);
    }

    query.all(self.connection()).await
  }

  pub async fn list_active_users_by_ids(
    &self,
    user_ids: Vec<String>,
  ) -> Result<Vec<users::Model>, DbErr> {
    if user_ids.is_empty() {
      return Ok(vec![]);
    }

    users::Entity::find()
      .filter(
        Condition::all()
          .add(users::Column::Id.is_in(user_ids))
          .add(users::Column::DeletedAt.is_null()),
      )
      .all(self.connection())
      .await
  }

  pub async fn list_active_users_paginated(
    &self,
    page: u64,
    page_size: u64,
  ) -> Result<(Vec<users::Model>, u64), DbErr> {
    let query = users::Entity::find()
      .filter(users::Column::DeletedAt.is_null())
      .order_by_desc(users::Column::CreatedAt);

    let paginator = query.paginate(self.connection(), page_size);
    let total = paginator.num_items().await?;
    let items = paginator.fetch_page(page.saturating_sub(1)).await?;
    Ok((items, total))
  }

  pub async fn insert_user(&self, active: users::ActiveModel) -> Result<users::Model, DbErr> {
    self.base.insert(active).await
  }

  pub async fn update_user_profile(
    &self,
    model: users::Model,
    username: Option<String>,
    email: Option<String>,
    password_hash: Option<String>,
    password_salt: Option<String>,
  ) -> Result<users::Model, DbErr> {
    let mut active = model.into_active_model();
    if let Some(username) = username {
      active.username = Set(username);
    }
    if let Some(email) = email {
      active.email = Set(email);
    }
    if let Some(password_hash) = password_hash {
      active.password = Set(password_hash);
    }
    if let Some(password_salt) = password_salt {
      active.password_salt = Set(password_salt);
    }
    active.updated_at = Set(Utc::now().into());
    self.base.update(active).await
  }

  pub async fn update_user_for_admin(
    &self,
    model: users::Model,
    username: Option<String>,
    email: Option<String>,
    password_hash: Option<String>,
    password_salt: Option<String>,
    status: Option<users::UserStatus>,
  ) -> Result<users::Model, DbErr> {
    let mut active = model.into_active_model();
    if let Some(username) = username {
      active.username = Set(username);
    }
    if let Some(email) = email {
      active.email = Set(email);
    }
    if let Some(password_hash) = password_hash {
      active.password = Set(password_hash);
    }
    if let Some(password_salt) = password_salt {
      active.password_salt = Set(password_salt);
    }
    if let Some(status) = status {
      active.status = Set(status);
    }
    active.updated_at = Set(Utc::now().into());
    self.base.update(active).await
  }

  pub async fn mark_user_deleted(
    &self,
    model: users::Model,
    deleted_at: Option<DateTimeWithTimeZone>,
    status: Option<users::UserStatus>,
  ) -> Result<users::Model, DbErr> {
    let mut active = model.into_active_model();
    active.deleted_at = Set(deleted_at);
    if let Some(status) = status {
      active.status = Set(status);
    }
    active.updated_at = Set(Utc::now().into());
    self.base.update(active).await
  }
}
