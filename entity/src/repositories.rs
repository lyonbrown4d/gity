use chrono::Utc;
use mr_ulid::Ulid;
use sea_orm::entity::prelude::*;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::Set;
pub use sea_orm::{
  ActiveModelBehavior, DeriveActiveEnum, DeriveRelation, EnumIter, Related, RelationDef,
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "repositories")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: String,

  pub organization_id: String,
  pub key: String,
  pub name: String,
  pub description: Option<String>,
  pub visibility: RepositoryVisibility,
  pub default_branch: String,
  pub created_by_user_id: String,

  pub created_at: DateTimeWithTimeZone,
  pub updated_at: DateTimeWithTimeZone,
  pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum RepositoryVisibility {
  #[sea_orm(string_value = "private")]
  Private,
  #[sea_orm(string_value = "internal")]
  Internal,
  #[sea_orm(string_value = "public")]
  Public,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::organizations::Entity",
    from = "Column::OrganizationId",
    to = "super::organizations::Column::Id",
    on_update = "NoAction",
    on_delete = "Cascade"
  )]
  Organization,
  #[sea_orm(
    belongs_to = "super::users::Entity",
    from = "Column::CreatedByUserId",
    to = "super::users::Column::Id",
    on_update = "NoAction",
    on_delete = "Cascade"
  )]
  CreatedBy,
}

impl Related<super::organizations::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Organization.def()
  }
}

impl Related<super::users::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::CreatedBy.def()
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

