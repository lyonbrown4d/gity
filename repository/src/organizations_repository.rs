use chrono::Utc;
use entity::organizations;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DbErr, EntityTrait, IntoActiveModel,
  QueryFilter, QueryOrder, Set, prelude::DateTimeWithTimeZone,
};

pub struct OrganizationsRepository;

impl OrganizationsRepository {
  pub async fn find_active_organization_by_id<C: ConnectionTrait>(
    conn: &C,
    organization_id: &str,
  ) -> Result<Option<organizations::Model>, DbErr> {
    organizations::Entity::find_by_id(organization_id.to_string())
      .filter(organizations::Column::DeletedAt.is_null())
      .one(conn)
      .await
  }

  pub async fn find_active_organization_by_key<C: ConnectionTrait>(
    conn: &C,
    key: &str,
  ) -> Result<Option<organizations::Model>, DbErr> {
    organizations::Entity::find()
      .filter(
        Condition::all()
          .add(organizations::Column::Key.eq(key.to_string()))
          .add(organizations::Column::DeletedAt.is_null()),
      )
      .one(conn)
      .await
  }

  pub async fn list_active_organizations_by_ids<C: ConnectionTrait>(
    conn: &C,
    organization_ids: Vec<String>,
  ) -> Result<Vec<organizations::Model>, DbErr> {
    if organization_ids.is_empty() {
      return Ok(vec![]);
    }

    organizations::Entity::find()
      .filter(
        Condition::all()
          .add(organizations::Column::Id.is_in(organization_ids))
          .add(organizations::Column::DeletedAt.is_null()),
      )
      .all(conn)
      .await
  }

  pub async fn list_active_organizations<C: ConnectionTrait>(
    conn: &C,
  ) -> Result<Vec<organizations::Model>, DbErr> {
    organizations::Entity::find()
      .filter(organizations::Column::DeletedAt.is_null())
      .order_by_desc(organizations::Column::CreatedAt)
      .all(conn)
      .await
  }

  pub async fn find_organization_by_key<C: ConnectionTrait>(
    conn: &C,
    key: &str,
  ) -> Result<Option<organizations::Model>, DbErr> {
    organizations::Entity::find()
      .filter(organizations::Column::Key.eq(key.to_string()))
      .one(conn)
      .await
  }

  pub async fn insert_organization<C: ConnectionTrait>(
    conn: &C,
    active: organizations::ActiveModel,
  ) -> Result<organizations::Model, DbErr> {
    active.insert(conn).await
  }

  pub async fn update_organization<C: ConnectionTrait>(
    conn: &C,
    model: organizations::Model,
    key: Option<String>,
    name: Option<String>,
    status: Option<organizations::OrgStatus>,
    deleted_at: Option<Option<DateTimeWithTimeZone>>,
  ) -> Result<organizations::Model, DbErr> {
    let mut active = model.into_active_model();
    if let Some(key) = key {
      active.key = Set(key);
    }
    if let Some(name) = name {
      active.name = Set(name);
    }
    if let Some(status) = status {
      active.status = Set(status);
    }
    if let Some(deleted_at) = deleted_at {
      active.deleted_at = Set(deleted_at);
    }
    active.updated_at = Set(Utc::now().into());
    active.update(conn).await
  }
}
