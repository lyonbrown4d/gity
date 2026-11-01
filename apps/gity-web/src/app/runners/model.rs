use crate::api;
use serde_json::{Value, json};

#[derive(Clone, PartialEq)]
pub(super) struct Runner {
  pub id: String,
  pub name: String,
  pub description: String,
  pub tags: Vec<String>,
  pub online: bool,
  pub active: bool,
  pub last_contact: String,
}
impl From<&Value> for Runner {
  fn from(value: &Value) -> Self {
    let name = api::text(value, "name");
    Self {
      id: api::text(value, "id"),
      name: if name.is_empty() {
        "runner".into()
      } else {
        name
      },
      description: api::text(value, "description"),
      tags: tags(&api::text(value, "tags")),
      online: api::text(value, "status").eq_ignore_ascii_case("online"),
      active: value["active"] == true,
      last_contact: api::text(value, "last_contact_at"),
    }
  }
}
pub(super) fn tags(value: &str) -> Vec<String> {
  value
    .split(',')
    .map(str::trim)
    .filter(|tag| !tag.is_empty())
    .map(str::to_owned)
    .collect()
}
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Stats {
  pub registered: usize,
  pub online: usize,
  pub offline: usize,
  pub active: usize,
}
impl Stats {
  pub fn from_runners(runners: &[Runner]) -> Self {
    let online = runners.iter().filter(|runner| runner.online).count();
    Self {
      registered: runners.len(),
      online,
      offline: runners.len() - online,
      active: runners.iter().filter(|runner| runner.active).count(),
    }
  }
}
pub(super) fn role_label(permissions: &Value) -> String {
  crate::ui::role_label(permissions)
}
pub(super) fn registration(
  name: &str,
  description: &str,
  tags: &str,
) -> Result<Value, &'static str> {
  if name.trim().is_empty() {
    return Err("Runner name is required");
  }
  Ok(json!({"name":name.trim(),"description":description.trim(),"tags":tags.trim()}))
}
pub(super) fn variable_body(
  key: &str,
  value: &str,
  masked: bool,
  protected: bool,
) -> Result<Value, &'static str> {
  let key = key.trim().to_uppercase();
  if key.is_empty() {
    return Err("CI variable key is required.");
  }
  if masked && value.trim().encode_utf16().count() < 8 {
    return Err("Masked variable values must be at least 8 characters.");
  }
  Ok(json!({"key":key,"value":value,"masked":masked,"protected":protected}))
}
pub(super) fn variable_guidance(masked: bool, protected: bool) -> &'static str {
  match (masked, protected) {
    (true, true) => {
      "This value will be hidden after save, redacted from logs, and marked for protected ref usage."
    }
    (true, false) => {
      "This value will be hidden after save and redacted from logs. Masked values must be at least 8 characters."
    }
    (false, true) => {
      "This value is visible to admins after entry but marked for protected ref usage; prefer masking deployment secrets too."
    }
    (false, false) => {
      "Unmasked variables are easier to diagnose but should only contain non-secret runtime configuration."
    }
  }
}
pub(super) fn variable_exposure(masked: bool, protected: bool) -> &'static str {
  match (masked, protected) {
    (true, true) => {
      "Hidden after save, redacted from logs, and intended for protected CI contexts."
    }
    (true, false) => "Hidden after save and redacted from logs; still injected into eligible jobs.",
    (false, true) => "Marked protected; avoid storing raw secrets unmasked.",
    (false, false) => "Visible variable; use for non-secret configuration only.",
  }
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
