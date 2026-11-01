use super::*;
use serde_json::json;

#[test]
fn release_creation_preserves_original_defaults_and_trims_user_fields() {
  assert_eq!(
    release_body(" v1.2.3 ", " ", " Notes ", " ", "stable", true),
    json!({"tag_name":"v1.2.3","name":"v1.2.3","description":"Notes","source_ref":"stable","create_tag":true})
  );
  assert_eq!(
    release_body("v1", " Name ", "", "feature/demo", "main", false)["source_ref"],
    "feature/demo"
  );
  assert_eq!(
    release_body("v1", "Name", "", "", "", false)["source_ref"],
    "main"
  );
}

#[test]
fn release_selection_and_asset_counts_stay_with_the_selected_release() {
  let rows = json!([
      {"release":{"id":"9007199254740993"},"links":[{"name":"first"}]},
      {"release":{"id":"9007199254740994"},"links":[{"name":"second"},{"name":"third"}]}
  ]);
  assert_eq!(link_count(&rows), 3);
  assert_eq!(links(&active(&rows, "9007199254740994")).len(), 2);
  assert!(active(&rows, "unknown").is_null());
}

#[test]
fn asset_links_only_expose_http_and_https_without_controls() {
  for value in [
    "https://example.com/artifact.tar.gz",
    "HTTP://example.com/manual#upgrade",
  ] {
    assert_eq!(asset_url(value), Some(value.into()));
  }
  for value in [
    "javascript:alert(1)",
    "data:text/html,unsafe",
    "//example.com",
    "https://",
    "https://example.com\n/unsafe",
    "https://bad host/",
  ] {
    assert!(asset_url(value).is_none(), "{value:?}");
  }
}

#[test]
fn tag_hashes_keep_original_eight_character_placeholder() {
  assert_eq!(short_sha(""), "--------");
  assert_eq!(short_sha("0123456789abcdef"), "01234567");
}
