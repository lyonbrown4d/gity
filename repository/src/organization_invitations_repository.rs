use chrono::{DateTime, Utc};
use entity::organization_invitations;
use sea_orm::sea_query::Expr;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DbErr, EntityTrait, IntoActiveModel,
  QueryFilter, Set,
};

pub struct OrganizationInvitationsRepository;

impl OrganizationInvitationsRepository {
  pub async fn find_pending_invitation_by_org_and_email<C: ConnectionTrait>(
    conn: &C,
    organization_id: &str,
    email: &str,
  ) -> Result<Option<organization_invitations::Model>, DbErr> {
    organization_invitations::Entity::find()
      .filter(
        Condition::all()
          .add(organization_invitations::Column::OrganizationId.eq(organization_id.to_string()))
          .add(organization_invitations::Column::Email.eq(email.to_string()))
          .add(
            organization_invitations::Column::Status
              .eq(organization_invitations::InvitationStatus::Pending),
          )
          .add(organization_invitations::Column::DeletedAt.is_null()),
      )
      .one(conn)
      .await
  }

  pub async fn insert_invitation<C: ConnectionTrait>(
    conn: &C,
    active: organization_invitations::ActiveModel,
  ) -> Result<organization_invitations::Model, DbErr> {
    active.insert(conn).await
  }

  pub async fn find_active_invitation_by_id<C: ConnectionTrait>(
    conn: &C,
    invitation_id: &str,
  ) -> Result<Option<organization_invitations::Model>, DbErr> {
    organization_invitations::Entity::find_by_id(invitation_id.to_string())
      .filter(organization_invitations::Column::DeletedAt.is_null())
      .one(conn)
      .await
  }

  pub async fn find_active_invitation_by_id_and_org<C: ConnectionTrait>(
    conn: &C,
    invitation_id: &str,
    organization_id: &str,
  ) -> Result<Option<organization_invitations::Model>, DbErr> {
    organization_invitations::Entity::find_by_id(invitation_id.to_string())
      .filter(
        Condition::all()
          .add(organization_invitations::Column::OrganizationId.eq(organization_id.to_string()))
          .add(organization_invitations::Column::DeletedAt.is_null()),
      )
      .one(conn)
      .await
  }

  pub async fn update_invitation<C: ConnectionTrait>(
    conn: &C,
    model: organization_invitations::Model,
    status: organization_invitations::InvitationStatus,
    accepted_by_user_id: Option<String>,
  ) -> Result<organization_invitations::Model, DbErr> {
    let mut active = model.into_active_model();
    active.status = Set(status);
    active.accepted_by_user_id = Set(accepted_by_user_id);
    active.updated_at = Set(Utc::now().into());
    active.update(conn).await
  }

  pub async fn expire_pending_invitations_before<C: ConnectionTrait>(
    conn: &C,
    now: DateTime<Utc>,
  ) -> Result<u64, DbErr> {
    let result = organization_invitations::Entity::update_many()
      .col_expr(
        organization_invitations::Column::Status,
        Expr::value(organization_invitations::InvitationStatus::Expired),
      )
      .col_expr(organization_invitations::Column::UpdatedAt, Expr::value(now))
      .filter(
        organization_invitations::Column::Status
          .eq(organization_invitations::InvitationStatus::Pending),
      )
      .filter(organization_invitations::Column::DeletedAt.is_null())
      .filter(organization_invitations::Column::ExpiresAt.lt(now))
      .exec(conn)
      .await?;

    Ok(result.rows_affected)
  }
}
