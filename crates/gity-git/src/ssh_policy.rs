use anyhow::{bail, Context, Result};
use git2::Oid;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const SSH_POLICY_ENV: &str = "GITY_SSH_POLICY";
const MAX_POLICY_BYTES: usize = 64 * 1024;

/// Server-created deployment-key grant. Its role is always developer (30),
/// independently of the creator's organization or administrator privileges.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SshPolicy {
    pub default_branch: String,
    pub rules: Vec<SshBranchRule>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SshBranchRule {
    pub branch_name: String,
    #[serde(default)]
    pub rule_type: SshRuleType,
    pub push_access_level: SshPushAccess,
    #[serde(default)]
    pub require_merge_request: bool,
    #[serde(default)]
    pub allow_force_push: bool,
    #[serde(default)]
    pub allow_delete: bool,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SshRuleType {
    #[default]
    Exact,
    Pattern,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SshPushAccess {
    #[serde(alias = "developers")]
    Developer,
    #[serde(alias = "maintainers")]
    Maintainer,
    #[serde(alias = "owners")]
    Owner,
    NoOne,
}
impl SshPolicy {
    pub fn validate(&self) -> Result<()> {
        if !git2::Reference::is_valid_name(&format!("refs/heads/{}", self.default_branch)) {
            bail!("invalid SSH policy default branch");
        }
        for rule in &self.rules {
            if rule.branch_name.is_empty() || rule.branch_name.len() > 255 {
                bail!("invalid SSH protected branch pattern");
            }
            match rule.rule_type {
                SshRuleType::Exact => {
                    if !git2::Reference::is_valid_name(&format!("refs/heads/{}", rule.branch_name))
                    {
                        bail!("invalid exact SSH protected branch");
                    }
                }
                SshRuleType::Pattern => {
                    glob::Pattern::new(&rule.branch_name)
                        .context("invalid SSH protected branch pattern")?;
                }
            }
        }
        Ok(())
    }
    pub fn to_environment(&self) -> Result<String> {
        self.validate()?;
        let json = serde_json::to_string(self)?;
        if json.len() > MAX_POLICY_BYTES {
            bail!("SSH branch policy exceeds its size limit");
        }
        Ok(json)
    }
    pub(crate) fn from_environment(json: &str) -> Result<Self> {
        if json.len() > MAX_POLICY_BYTES {
            bail!("SSH branch policy exceeds its size limit");
        }
        let policy: Self = serde_json::from_str(json).context("invalid SSH branch policy")?;
        policy.validate()?;
        Ok(policy)
    }
    fn effective_rule(&self, branch: &str) -> Option<&SshBranchRule> {
        self.rules
            .iter()
            .find(|rule| rule.rule_type == SshRuleType::Exact && rule.branch_name == branch)
            .or_else(|| {
                self.rules
                    .iter()
                    .filter(|rule| {
                        rule.rule_type == SshRuleType::Pattern
                            && glob::Pattern::new(&rule.branch_name)
                                .is_ok_and(|pattern| pattern.matches(branch))
                    })
                    .min_by_key(|rule| &rule.branch_name)
            })
    }
    /// Called by the native update hook for every proposed ref, including new
    /// branches not present when the SSH session began.
    pub fn check_update(
        &self,
        repo: &Path,
        git_bin: &Path,
        reference: &str,
        old: &str,
        new: &str,
    ) -> Result<()> {
        self.validate()?;
        if !git2::Reference::is_valid_name(reference) {
            bail!("invalid ref update");
        }
        let old = Oid::from_str(old).context("invalid old object ID")?;
        let new = Oid::from_str(new).context("invalid new object ID")?;
        let Some(branch) = reference.strip_prefix("refs/heads/") else {
            if reference.starts_with("refs/tags/") {
                return Ok(());
            }
            bail!("reference namespace not writable");
        };
        if branch == self.default_branch && new.is_zero() {
            bail!("cannot delete default branch");
        }
        let Some(rule) = self.effective_rule(branch) else {
            return Ok(());
        };
        if !matches!(rule.push_access_level, SshPushAccess::Developer) {
            bail!("protected branch role requirement not met");
        }
        if new.is_zero() {
            if !rule.allow_delete {
                bail!("protected branch cannot be deleted");
            }
            return Ok(());
        }
        if rule.require_merge_request {
            bail!("protected branch requires a merge request");
        }
        if !old.is_zero() && !rule.allow_force_push {
            crate::hooks::check_fast_forward(repo, git_bin, reference, old, new)?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "ssh_policy_tests.rs"]
mod tests;
