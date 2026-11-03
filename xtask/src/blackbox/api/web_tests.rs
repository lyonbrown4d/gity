use super::*;

#[test]
fn vite_entry_and_lazy_chunks_include_css_and_ignore_non_assets() {
  let html =
    r#"<script src="/assets/index-123.js"></script><link href="/assets/index-123.css?x=1">"#;
  assert_eq!(
    static_references(html),
    BTreeSet::from([
      "/assets/index-123.js".into(),
      "/assets/index-123.css".into(),
    ])
  );
  let chunks = r#"import './runtime-123.js';const d=["assets/editor-123.js","assets/editor-123.css"];const text="return value.js";"#;
  assert_eq!(
    static_references(chunks),
    BTreeSet::from([
      "./runtime-123.js".into(),
      "assets/editor-123.js".into(),
      "assets/editor-123.css".into(),
    ])
  );
  assert!(starts_with_html(b" <!doctype html><html>fallback"));
  assert!(!starts_with_html(b"export const x = 1;"));
}
