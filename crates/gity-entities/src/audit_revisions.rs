use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "audit_revisions")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: i64,
  pub created_at: chrono::NaiveDateTime,
  pub actor: Option<String>,
  pub tenant: Option<String>,
  pub reason: Option<String>,
  pub metadata: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {
  fn new() -> Self {
    Self {
      id: sea_orm::ActiveValue::Set(gity_ids::new_id()),
      ..<Self as ActiveModelTrait>::default()
    }
  }
}
