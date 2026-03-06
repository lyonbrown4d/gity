use axum::http::StatusCode;
use entity::organization_members;
use repository::OrganizationMembersRepository;
use sea_orm::DatabaseConnection;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequiredOrganizationRole {
  Member,
  Owner,
}

#[derive(Debug)]
pub struct AccessError {
  pub status: StatusCode,
  pub message: String,
}

pub async fn require_organization_role(
  db: &DatabaseConnection,
  user_id: &str,
  organization_id: &str,
  required: RequiredOrganizationRole,
) -> Result<organization_members::Model, AccessError> {
  let membership = OrganizationMembersRepository::new(db.clone())
    .find_active_membership(user_id, organization_id)
    .await
    .map_err(|err| AccessError {
      status: StatusCode::INTERNAL_SERVER_ERROR,
      message: format!("failed to load organization membership: {err}"),
    })?;

  let membership = match membership {
    Some(membership) => membership,
    None => {
      return Err(AccessError {
        status: StatusCode::FORBIDDEN,
        message: "you are not a member of this organization".to_string(),
      });
    }
  };

  let allowed = match required {
    RequiredOrganizationRole::Member => true,
    RequiredOrganizationRole::Owner => membership.role == organization_members::MemberRole::Owner,
  };

  if !allowed {
    return Err(AccessError {
      status: StatusCode::FORBIDDEN,
      message: "insufficient organization permission".to_string(),
    });
  }

  Ok(membership)
}

pub fn parse_member_role(role: Option<&str>) -> Option<organization_members::MemberRole> {
  match role.unwrap_or("member").to_ascii_lowercase().as_str() {
    "owner" => Some(organization_members::MemberRole::Owner),
    "member" => Some(organization_members::MemberRole::Member),
    _ => None,
  }
}

pub fn member_role_to_string(role: organization_members::MemberRole) -> String {
  match role {
    organization_members::MemberRole::Owner => "owner".to_string(),
    organization_members::MemberRole::Member => "member".to_string(),
  }
}
