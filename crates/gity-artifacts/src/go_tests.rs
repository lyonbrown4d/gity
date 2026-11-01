use super::*;

#[test]
fn module_path_escaping_roundtrips_uppercase_and_bang() {
  let path = "github.com/Azure/SDK!v2";
  let escaped = go_escape_path(path);
  assert_eq!(escaped, "github.com/!azure/!s!d!k!!v2");
  assert_eq!(go_unescape_path(&escaped).unwrap(), path);
}

#[test]
fn rejects_unescaped_or_malformed_module_path() {
  assert!(go_unescape_path("github.com/Upper/Repo").is_err());
  assert!(go_unescape_path("github.com/!7invalid").is_err());
}
