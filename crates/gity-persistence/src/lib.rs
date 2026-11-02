//! Typed cross-context persistence primitives. No table-name/JSON CRUD API.
mod audit;
mod counters;
pub use audit::{audit_deploy_key, audit_member, audit_protection, audit_rule, audit_variable};
pub use counters::{CounterKind, next_iid};

/// Adapt persisted scalar models to existing HTTP presenters at the boundary.
/// Public response presenters must explicitly exclude credentials and other secrets.
pub fn model_value(model: impl serde::Serialize) -> serde_json::Value {
  let mut value = serde_json::to_value(model).expect("typed scalar entity model must serialize");
  gity_core::normalize_timestamps(&mut value);
  value
}
