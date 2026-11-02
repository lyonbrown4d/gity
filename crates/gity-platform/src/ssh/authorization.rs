use super::{
  keys::{self, KeyIdentity},
  transfer::Authorized,
};
use crate::{AppState, access, projects, repository};
use gity_core::Principal;
use gity_git::{GitRpc, SshPolicy};
use serde_json::Value;

pub(super) async fn authorize(
  state: &AppState,
  fingerprint: &str,
  path: &str,
  service: GitRpc,
) -> anyhow::Result<Authorized> {
  let project = state
    .projects
    .active_by_path(path)
    .await?
    .filter(|project| project.status == "active")
    .map(gity_persistence::model_value)
    .ok_or_else(|| anyhow::anyhow!("repository unavailable"))?;
  let id = projects::id(&project["id"])?;
  // Re-resolve on every exec, including channels on already authenticated sessions.
  let identity = keys::authorized(state, fingerprint)
    .await?
    .ok_or_else(|| anyhow::anyhow!("SSH key unavailable"))?;
  let (level, actor_id, personal, deploy) = match identity {
    KeyIdentity::Personal(key, user) => {
      let actor = Principal {
        user_id: user.id,
        username: user.username,
        is_super_admin: user.is_super_admin,
      };
      let level = access::level(state, &actor, &project).await?;
      if service == GitRpc::ReceivePack {
        anyhow::ensure!(level >= 30, "insufficient repository push permissions");
      } else {
        anyhow::ensure!(
          level > 0 || project["visibility"] == "public" || project["visibility"] == "internal",
          "insufficient repository read permissions"
        );
      }
      (level, Some(user.id), Some((user.id, key.id)), None)
    }
    KeyIdentity::Deploy(keys) => {
      let key = keys
        .into_iter()
        .find(|key| key["project_id"] == project["id"])
        .ok_or_else(|| anyhow::anyhow!("key is not authorized for this repository"))?;
      if service == GitRpc::ReceivePack {
        anyhow::ensure!(
          key["can_push"]
            .as_bool()
            .unwrap_or_else(|| key["can_push"].as_i64() == Some(1)),
          "deploy key is read-only"
        );
      }
      (30, None, None, Some(projects::id(&key["id"])?))
    }
  };
  let policy = if service == GitRpc::ReceivePack {
    let mut rules = state
      .repository_access
      .protections(id)
      .await?
      .into_iter()
      .map(gity_persistence::model_value)
      .collect::<Vec<_>>();
    for rule in &mut rules {
      for field in ["require_merge_request", "allow_force_push", "allow_delete"] {
        rule[field] = Value::Bool(
          rule[field]
            .as_bool()
            .unwrap_or_else(|| rule[field].as_i64() == Some(1)),
        );
      }
    }
    state
      .git
      .ensure_update_hook(
        &repository::repo(&project),
        state
          .settings
          .git_hook
          .clone()
          .unwrap_or(std::env::current_exe()?),
      )
      .await?;
    Some(SshPolicy {
      default_branch: repository::default_branch(&project).into(),
      rules: serde_json::from_value(Value::Array(rules))?,
      actor_level: level,
    })
  } else {
    None
  };
  if let Some((user, key)) = personal {
    anyhow::ensure!(
      state.identity.touch_personal_ssh_key(user, key).await?,
      "SSH key revoked"
    );
  }
  if let Some(key) = deploy {
    state.repository_access.touch_deploy_key(key).await?;
  }
  Ok(Authorized {
    project,
    service,
    policy,
    actor_id,
    protocol: None,
  })
}

#[cfg(test)]
#[path = "authorization_tests.rs"]
mod tests;
