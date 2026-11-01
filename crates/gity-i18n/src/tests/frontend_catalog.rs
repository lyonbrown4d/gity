use crate::assets::CHUNKS;
use regex::Regex;
use serde_json::Value;
use std::{
  collections::{BTreeSet, HashSet},
  path::Path,
};

#[test]
fn static_frontend_translation_calls_and_component_labels_have_shared_assets() {
  let keys = CHUNKS
    .iter()
    .flat_map(|chunk| {
      let value: Value = serde_json::from_str(chunk).unwrap();
      value
        .as_object()
        .unwrap()
        .keys()
        .filter(|key| key.as_str() != "_version")
        .cloned()
        .collect::<Vec<_>>()
    })
    .collect::<HashSet<_>>();
  let call = Regex::new(r#"\bt\(\s*("(?:\\.|[^"\\])*")\s*,?\s*\)"#).unwrap();
  let prop = Regex::new(
    r#"\b(?:title|description|label|subtitle|caption|empty_text)\s*=\s*("(?:\\.|[^"\\])*")"#,
  )
  .unwrap();
  let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/gity-web/src");
  let mut missing = BTreeSet::new();
  visit(&root, &keys, &[call, prop], &mut missing);
  assert!(
    missing.is_empty(),
    "UI translation keys missing from shared assets: {missing:?}"
  );
}
fn visit(path: &Path, keys: &HashSet<String>, patterns: &[Regex], missing: &mut BTreeSet<String>) {
  for entry in std::fs::read_dir(path).unwrap() {
    let entry = entry.unwrap();
    let path = entry.path();
    if path.is_dir() {
      if entry.file_name() != "tests" {
        visit(&path, keys, patterns, missing);
      }
    } else if path.extension().is_some_and(|extension| extension == "rs")
      && !path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .ends_with("_tests.rs")
    {
      let text = std::fs::read_to_string(&path).unwrap();
      for pattern in patterns {
        for captures in pattern.captures_iter(&text) {
          if let Ok(key) = serde_json::from_str::<String>(&captures[1])
            && !key.is_empty()
            && !keys.contains(&key)
          {
            missing.insert(key);
          }
        }
      }
    }
  }
}
