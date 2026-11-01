use gity_git_wire::RepositoryKey;
#[test]
fn repository_keys_are_stable_project_identifiers() {
  assert_eq!(RepositoryKey::for_project(42).unwrap().as_str(), "42.git");
  for key in [
    "../42.git",
    "team/project.git",
    "/42.git",
    "42.git/objects",
    "0.git",
    "042.git",
    "42\0.git",
  ] {
    assert!(RepositoryKey::try_from(key.to_owned()).is_err(), "{key}");
  }
}
