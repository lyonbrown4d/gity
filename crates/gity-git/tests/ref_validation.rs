use gity_git::GitService;
#[test]
fn initial_branch_rejects_unbounded_or_unwritable_reference_names() {
  let git = GitService::new("unused-repository-root", "git");
  for value in [
    String::new(),
    "-option".into(),
    "bad branch".into(),
    "x".repeat(241),
    "x/".repeat(600),
  ] {
    assert!(git.validate_initial_branch(&value).is_err());
  }
  assert!(git.validate_initial_branch("feature/platform").is_ok());
  assert!(git.validate_initial_branch("main").is_ok());
}
