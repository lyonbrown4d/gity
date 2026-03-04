use argon2::password_hash::phc::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::Utc;
use domain::user::{CreateUser, UserViewObject};
use mr_ulid::Ulid;
use sea_orm::Set;
use sea_orm::entity::prelude::*;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::prelude::async_trait::async_trait;
pub use sea_orm::{
  ActiveModelBehavior, DeriveActiveEnum, DeriveRelation, EnumIter, Related, RelationDef,
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: String,

  pub username: String,
  pub email: String,
  pub password: String,
  pub password_salt: String,

  pub status: UserStatus,

  pub created_at: DateTimeWithTimeZone,
  pub updated_at: DateTimeWithTimeZone,
  pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum UserStatus {
  #[sea_orm(string_value = "active")]
  Active,

  #[sea_orm(string_value = "disabled")]
  Disabled,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "super::organization_members::Entity")]
  OrganizationMembers,
}

impl Related<super::organization_members::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::OrganizationMembers.def()
  }
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
  fn new() -> Self {
    let now = Utc::now().into();
    Self {
      id: Set(Ulid::new().to_string()),
      created_at: Set(now),
      updated_at: Set(now),
      ..ActiveModelTrait::default()
    }
  }
}

impl From<CreateUser> for ActiveModel {
  fn from(dto: CreateUser) -> Self {
    let now = Utc::now().into();
    let hashed_password = Model::hash_password(&dto.password);
    ActiveModel {
      username: Set(dto.username),
      email: Set(dto.email),
      password: Set(hashed_password.0),
      password_salt: Set(hashed_password.1.to_string()),
      status: Set(UserStatus::Active),
      created_at: Set(now),
      updated_at: Set(now),
      ..Default::default()
    }
  }
}

impl From<Model> for UserViewObject {
  fn from(model: Model) -> Self {
    UserViewObject {
      id: model.id,
      username: model.username,
    }
  }
}

impl Model {
  pub fn hash_password(password: &str) -> (String, SaltString) {
    let argon2 = Argon2::default();
    let salt = SaltString::generate();
    let hashed = argon2
      .hash_password_with_salt(password.as_bytes(), salt.as_bytes())
      .unwrap()
      .to_string();
    (hashed, salt)
  }

  pub fn verify_password(password: &str, hash: &str) -> bool {
    let parsed_hash = PasswordHash::new(hash).unwrap();
    Argon2::default()
      .verify_password(password.as_bytes(), &parsed_hash)
      .is_ok()
  }
}
