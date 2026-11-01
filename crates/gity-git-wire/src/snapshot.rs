use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "target", rename_all = "snake_case")]
pub enum RepositoryHead {
  Symbolic(String),
  Detached(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceSnapshot {
  pub head: RepositoryHead,
  pub refs: BTreeMap<String, String>,
}
impl ReferenceSnapshot {
  pub fn fingerprint(&self) -> String {
    let bytes = serde_json::to_vec(self).expect("reference snapshot contains serializable strings");
    format!("{:x}", Sha256::digest(bytes))
  }
}
