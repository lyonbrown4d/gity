use chrono::Utc;
use entity::users;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DbErr, EntityTrait, IntoActiveModel,
  QueryFilter, QueryOrder, QuerySelect, Set,
};

pub struct UsersRepository;

impl UsersRepository {
  pub async fn find_user_by_username_or_email<C: ConnectionTrait>(
    conn: &C,
    identity: &str,
  ) -> Result<Option<users::Model>, DbErr> {
    users::Entity::find()
      .filter(
        Condition::any()
          .add(users::Column::Username.eq(identity.to_string()))
          .add(users::Column::Email.eq(identity.to_string())),
      )
      .one(conn)
      .await
  }

  pub async fn find_duplicate_user_by_username_or_email<C: ConnectionTrait>(
    conn: &C,
    username: &str,
    email: &str,
  ) -> Result<Option<users::Model>, DbErr> {
    users::Entity::find()
      .filter(
        Condition::any()
          .add(users::Column::Username.eq(username.to_string()))
          .add(users::Column::Email.eq(email.to_string())),
      )
      .one(conn)
      .await
  }

  pub async fn find_active_user_by_id<C: ConnectionTrait>(
    conn: &C,
    user_id: &str,
  ) -> Result<Option<users::Model>, DbErr> {
    users::Entity::find_by_id(user_id.to_string())
      .filter(users::Column::DeletedAt.is_null())
      .one(conn)
      .await
  }

  pub async fn list_active_users<C: ConnectionTrait>(
    conn: &C,
    limit: Option<u64>,
  ) -> Result<Vec<users::Model>, DbErr> {
    let mut query = users::Entity::find()
      .filter(users::Column::DeletedAt.is_null())
      .order_by_desc(users::Column::CreatedAt);

    if let Some(limit) = limit {
      query = query.limit(limit);
    }

    query.all(conn).await
  }

  pub async fn list_active_users_by_ids<C: ConnectionTrait>(
    conn: &C,
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
      .all(conn)
      .await
  }

  pub async fn insert_user<C: ConnectionTrait>(
    conn: &C,
    active: users::ActiveModel,
  ) -> Result<users::Model, DbErr> {
    active.insert(conn).await
  }

  pub async fn update_user_profile<C: ConnectionTrait>(
    conn: &C,
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
    active.update(conn).await
  }
}
