use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Storage keys remain stable when project names change.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RepositoryKey(String);
impl RepositoryKey {
  pub fn for_project(id: u64) -> anyhow::Result<Self> {
    anyhow::ensure!(id > 0, "invalid project identifier");
    Ok(Self(format!("{id}.git")))
  }
  pub fn as_str(&self) -> &str {
    &self.0
  }
}
impl TryFrom<String> for RepositoryKey {
  type Error = anyhow::Error;
  fn try_from(value: String) -> Result<Self, Self::Error> {
    let id = value
      .strip_suffix(".git")
      .ok_or_else(|| anyhow::anyhow!("invalid repository key"))?;
    let parsed: u64 = id.parse()?;
    let key = Self::for_project(parsed)?;
    anyhow::ensure!(key.0 == value, "noncanonical repository key");
    Ok(key)
  }
}
impl From<RepositoryKey> for String {
  fn from(key: RepositoryKey) -> Self {
    key.0
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum GitOperation {
  InitBare {
    initial_branch: String,
  },
  FileExists {
    revision: String,
    default: String,
    path: String,
  },
  ListBranches {
    default: String,
  },
  ListTags,
  ListTree {
    revision: String,
    path: String,
    default: String,
  },
  GetBlob {
    revision: String,
    path: String,
    default: String,
    max_bytes: usize,
  },
  GetReadme {
    revision: String,
    default: String,
  },
  ListCommits {
    revision: String,
    default: String,
    limit: usize,
  },
  ResolveRevision {
    revision: String,
    default: String,
  },
  Search {
    revision: String,
    default: String,
    parameters: serde_json::Value,
  },
  AnalyzeLanguages {
    revision: String,
    default: String,
  },
  CreateBranch {
    name: String,
    source: String,
  },
  DeleteBranch {
    name: String,
  },
  CreateTag {
    name: String,
    source: String,
  },
  DeleteTag {
    name: String,
  },
  CreateFileCommit {
    input: serde_json::Value,
  },
  MergeBranches {
    input: serde_json::Value,
  },
  DiffBranches {
    target: String,
    source: String,
  },
  Maintenance {
    kind: Maintenance,
  },
  InstallManagedHook,
  ReferenceSnapshot,
  SyncReplica {
    source: crate::NodeEndpoint,
    source_epoch: u64,
  },
  FenceRepository {
    next_epoch: u64,
  },
  ActivatePrimary {
    epoch: u64,
  },
}
impl GitOperation {
  pub fn is_control(&self) -> bool {
    matches!(
      self,
      Self::ReferenceSnapshot
        | Self::SyncReplica { .. }
        | Self::FenceRepository { .. }
        | Self::ActivatePrimary { .. }
    )
  }
  pub fn is_write(&self) -> bool {
    matches!(
      self,
      Self::InitBare { .. }
        | Self::CreateBranch { .. }
        | Self::DeleteBranch { .. }
        | Self::CreateTag { .. }
        | Self::DeleteTag { .. }
        | Self::CreateFileCommit { .. }
        | Self::MergeBranches { .. }
        | Self::Maintenance { .. }
        | Self::InstallManagedHook
    )
  }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Maintenance {
  GarbageCollect,
  Repack,
  UpdateServerInfo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TransferKind {
  Bundle,
  Http {
    receive: bool,
    advertise: bool,
    environment: BTreeMap<String, String>,
  },
  Ssh {
    receive: bool,
    protocol: Option<String>,
    policy: Option<serde_json::Value>,
  },
  Archive {
    revision: String,
  },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransferOpen {
  pub epoch: u64,
  pub repository: RepositoryKey,
  pub kind: TransferKind,
}
