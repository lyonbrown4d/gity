use super::*;

const ZERO: &str = "0000000000000000000000000000000000000000";
const ONE: &str = "1111111111111111111111111111111111111111";
fn policy(rules: Vec<SshBranchRule>) -> SshPolicy {
  SshPolicy {
    default_branch: "main".into(),
    rules,
  }
}
fn rule(branch: &str, kind: SshRuleType, access: SshPushAccess) -> SshBranchRule {
  SshBranchRule {
    branch_name: branch.into(),
    rule_type: kind,
    push_access_level: access,
    require_merge_request: false,
    allow_force_push: false,
    allow_delete: false,
  }
}
fn check(policy: &SshPolicy, reference: &str, deleting: bool) -> Result<()> {
  policy.check_update(
    Path::new("."),
    Path::new("git"),
    reference,
    if deleting { ONE } else { ZERO },
    if deleting { ZERO } else { ONE },
  )
}

#[test]
fn default_branch_and_ref_namespaces_are_enforced_without_protection_rows() {
  let policy = policy(vec![]);
  assert!(check(&policy, "refs/heads/main", true)
    .unwrap_err()
    .to_string()
    .contains("default branch"));
  assert!(check(&policy, "refs/heads/feature", true).is_ok());
  assert!(check(&policy, "refs/tags/v1", false).is_ok());
  assert!(check(&policy, "refs/tags/v1", true).is_ok());
  for reference in [
    "refs/replace/object",
    "refs/notes/review",
    "refs/heads/../escape",
    "HEAD",
  ] {
    assert!(check(&policy, reference, false).is_err(), "{reference}");
  }
}

#[test]
fn new_branches_follow_patterns_and_never_inherit_key_creator_privileges() {
  for access in [
    SshPushAccess::Maintainer,
    SshPushAccess::Owner,
    SshPushAccess::NoOne,
  ] {
    let mut protected = rule("release/*", SshRuleType::Pattern, access);
    protected.allow_delete = true;
    let policy = policy(vec![protected]);
    assert!(check(&policy, "refs/heads/release/new", false).is_err());
    assert!(check(&policy, "refs/heads/release/new", true).is_err());
  }
  let mut protected = rule("release/*", SshRuleType::Pattern, SshPushAccess::Developer);
  protected.require_merge_request = true;
  assert!(check(
    &policy(vec![protected.clone()]),
    "refs/heads/release/new",
    false
  )
  .unwrap_err()
  .to_string()
  .contains("merge request"));
  protected.require_merge_request = false;
  assert!(check(
    &policy(vec![protected.clone()]),
    "refs/heads/release/new",
    false
  )
  .is_ok());
  assert!(check(
    &policy(vec![protected.clone()]),
    "refs/heads/release/new",
    true
  )
  .is_err());
  protected.allow_delete = true;
  protected.require_merge_request = true;
  assert!(check(&policy(vec![protected]), "refs/heads/release/new", true).is_ok());
}

#[test]
fn exact_protection_precedes_patterns_and_patterns_use_lexical_order() {
  let broad = rule("*", SshRuleType::Pattern, SshPushAccess::NoOne);
  let specific = rule("feature/*", SshRuleType::Pattern, SshPushAccess::Developer);
  assert!(check(
    &policy(vec![specific.clone(), broad.clone()]),
    "refs/heads/feature/new",
    false
  )
  .is_err());
  let exact = rule("feature/new", SshRuleType::Exact, SshPushAccess::Developer);
  assert!(check(
    &policy(vec![specific, broad, exact]),
    "refs/heads/feature/new",
    false
  )
  .is_ok());
}

#[test]
fn policy_environment_rejects_malformed_oversized_and_invalid_rules() {
  let json = policy(vec![]).to_environment().unwrap();
  assert!(SshPolicy::from_environment(&json).is_ok());
  for invalid in [
    "",
    "{}",
    "{invalid}",
    r#"{"default_branch":"main","rules":[],"role":100}"#,
  ] {
    assert!(SshPolicy::from_environment(invalid).is_err());
  }
  assert!(SshPolicy::from_environment(&" ".repeat(MAX_POLICY_BYTES + 1)).is_err());
  let mut invalid = policy(vec![]);
  invalid.default_branch = "../escape".into();
  assert!(invalid.to_environment().is_err());
  assert!(policy(vec![rule(
    "[",
    SshRuleType::Pattern,
    SshPushAccess::Developer
  )])
  .to_environment()
  .is_err());
  let row = serde_json::json!({"branch_name":"main","rule_type":"exact","push_access_level":"developers","allow_delete":false,"id":"42","project_id":"11"});
  let decoded: SshBranchRule = serde_json::from_value(row).unwrap();
  assert!(matches!(
    decoded.push_access_level,
    SshPushAccess::Developer
  ));
}

fn commit(repo: &git2::Repository, parents: &[Oid], message: &str) -> Oid {
  let tree_id = repo.treebuilder(None).unwrap().write().unwrap();
  let tree = repo.find_tree(tree_id).unwrap();
  let signature = git2::Signature::now("SSH test", "ssh@example.test").unwrap();
  let parents = parents
    .iter()
    .map(|id| repo.find_commit(*id).unwrap())
    .collect::<Vec<_>>();
  repo
    .commit(
      None,
      &signature,
      &signature,
      message,
      &tree,
      &parents.iter().collect::<Vec<_>>(),
    )
    .unwrap()
}

#[test]
fn protected_fast_forwards_use_real_git_ancestry_and_reject_divergence() {
  let directory = tempfile::tempdir().unwrap();
  let repo = git2::Repository::init_bare(directory.path()).unwrap();
  let first = commit(&repo, &[], "first");
  let second = commit(&repo, &[first], "second");
  let divergent = commit(&repo, &[first], "divergent");
  let mut protected = rule("main", SshRuleType::Exact, SshPushAccess::Developer);
  let check = |rule: SshBranchRule, old: Oid, new: Oid| {
    policy(vec![rule]).check_update(
      directory.path(),
      Path::new("git"),
      "refs/heads/main",
      &old.to_string(),
      &new.to_string(),
    )
  };
  assert!(check(protected.clone(), first, second).is_ok());
  assert!(check(protected.clone(), second, divergent)
    .unwrap_err()
    .to_string()
    .contains("force push"));
  protected.allow_force_push = true;
  assert!(check(protected, second, divergent).is_ok());
}
