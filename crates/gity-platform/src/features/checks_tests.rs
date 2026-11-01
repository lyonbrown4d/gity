use super::*;
use crate::features::codeowners::parse_owner;
#[test]
fn approval_checks_exclude_ineligible_approvers_and_enforce_threshold() {
  let rule = json!({"id":12,"name":"Security","target_branch":"main","approvals_required":2});
  let blocked = rule_check(&rule, &[2, 3], &[2, 4]);
  assert_eq!(blocked["approval_count"], 1);
  assert_eq!(blocked["satisfied"], false);
  assert_eq!(rule_check(&rule, &[2, 3], &[2, 3])["satisfied"], true);
}
#[test]
fn glob_patterns_and_escaped_codeowner_fields_preserve_last_match_semantics() {
  assert!(matches("release/*", "release/v1"));
  assert!(!matches("release/*", "release/x/y"));
  assert!(matches("**/*.rs", "src/main.rs"));
  assert!(matches("**/*.rs", "main.rs"));
  assert!(matches("release/v[0-9]", "release/v2"));
  assert!(!matches("release/v[!0-9]", "release/v2"));
  assert_eq!(
    parse_owner(r"docs/my\ file.md @Alice @bob # note"),
    Some(("docs/my file.md".into(), vec!["alice".into(), "bob".into()]))
  );
  assert!(parse_owner("# ignored").is_none());
  assert!(parse_owner("[Section]").is_none());
}
