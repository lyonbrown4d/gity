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
#[sea_orm(table_name = "organization_invitations")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: String,

  pub organization_id: String,
  pub email: String,
  pub role: InvitationRole,
  pub status: InvitationStatus,

  pub invited_by_user_id: String,
  pub accepted_by_user_id: Option<String>,
  pub expires_at: Option<DateTimeWithTimeZone>,

  pub created_at: DateTimeWithTimeZone,
  pub updated_at: DateTimeWithTimeZone,
  pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum InvitationRole {
  #[sea_orm(string_value = "owner")]
  Owner,
  #[sea_orm(string_value = "member")]
  Member,
}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum InvitationStatus {
  #[sea_orm(string_value = "pending")]
  Pending,
  #[sea_orm(string_value = "accepted")]
  Accepted,
  #[sea_orm(string_value = "revoked")]
  Revoked,
  #[sea_orm(string_value = "expired")]
  Expired,
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
    from = "Column::InvitedByUserId",
    to = "super::users::Column::Id",
    on_update = "NoAction",
    on_delete = "Cascade"
  )]
  InvitedBy,
}

impl Related<super::organizations::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Organization.def()
  }
}

impl Related<super::users::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::InvitedBy.def()
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
