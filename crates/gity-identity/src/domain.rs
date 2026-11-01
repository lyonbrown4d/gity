#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserIdentity {
  pub id: i64,
  pub username: String,
  pub status: String,
  pub is_super_admin: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UserProfileChanges {
  pub username: Option<String>,
  pub display_name: Option<String>,
  pub email: Option<Option<String>>,
  pub is_super_admin: Option<bool>,
  pub status: Option<String>,
  pub password_hash: Option<String>,
}

impl UserIdentity {
  pub fn is_active(&self) -> bool {
    self.status == "active"
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MembershipRole {
  None,
  Guest,
  Reporter,
  Developer,
  Maintainer,
  Owner,
  Administrator,
}

impl MembershipRole {
  pub fn parse(value: &str) -> Self {
    match value {
      "guest" => Self::Guest,
      "reporter" => Self::Reporter,
      "developer" => Self::Developer,
      "maintainer" => Self::Maintainer,
      "owner" => Self::Owner,
      _ => Self::None,
    }
  }

  pub fn level(self) -> i64 {
    match self {
      Self::None => 0,
      Self::Guest => 10,
      Self::Reporter => 20,
      Self::Developer => 30,
      Self::Maintainer => 40,
      Self::Owner => 50,
      Self::Administrator => 100,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectAccess {
  pub project_id: i64,
  pub organization_id: i64,
  pub visibility: String,
  pub membership: MembershipRole,
}

impl ProjectAccess {
  pub fn effective_role(
    &self,
    organization_role: MembershipRole,
    administrator: bool,
  ) -> MembershipRole {
    if administrator {
      MembershipRole::Administrator
    } else {
      self.membership.max(organization_role)
    }
  }

  pub fn is_readable_by(&self, organization_role: MembershipRole, administrator: bool) -> bool {
    administrator
      || self.visibility == "public"
      || self.visibility == "internal"
      || self.effective_role(organization_role, false) != MembershipRole::None
  }
}
