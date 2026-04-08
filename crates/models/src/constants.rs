pub mod namespace_kind {
  pub const USER: &str = "user";
  pub const GROUP: &str = "group";
}

pub mod visibility_level {
  pub const PRIVATE: &str = "private";
  pub const INTERNAL: &str = "internal";
  pub const PUBLIC: &str = "public";
}

pub mod member_role {
  pub const GUEST: &str = "guest";
  pub const REPORTER: &str = "reporter";
  pub const DEVELOPER: &str = "developer";
  pub const MAINTAINER: &str = "maintainer";
  pub const OWNER: &str = "owner";
}

pub mod record_state {
  pub const ACTIVE: &str = "active";
  pub const BLOCKED: &str = "blocked";
  pub const ARCHIVED: &str = "archived";
}

pub mod issue_state {
  pub const OPEN: &str = "open";
  pub const CLOSED: &str = "closed";
}

pub mod invitation_state {
  pub const PENDING: &str = "pending";
  pub const ACCEPTED: &str = "accepted";
  pub const REVOKED: &str = "revoked";
  pub const EXPIRED: &str = "expired";
}
