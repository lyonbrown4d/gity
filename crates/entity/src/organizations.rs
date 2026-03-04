use chrono::Utc;
use mr_ulid::Ulid;
use sea_orm::Set;
use sea_orm::entity::prelude::*;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::prelude::async_trait::async_trait;
pub use sea_orm::{
  ActiveModelBehavior, DeriveActiveEnum, DeriveRelation, EnumIter, Related, RelationDef,
};
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "organizations")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: String,

  #[sea_orm(unique)]
  pub key: String,

  pub name: String,
  pub status: OrgStatus,

  pub created_at: DateTimeWithTimeZone,
  pub updated_at: DateTimeWithTimeZone,
  pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum OrgStatus {
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
