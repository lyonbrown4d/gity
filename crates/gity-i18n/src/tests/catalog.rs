use super::{assets::CHUNKS, normalize_locale, translate, translate_message};
use regex::Regex;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn shared_assets_preserve_original_keys_and_placeholder_parity() {
  let mut messages = BTreeMap::new();
  let marker = Regex::new(r"%\{([A-Za-z_][A-Za-z_0-9]*)\}").unwrap();
  for source in CHUNKS {
    let value: Value = serde_json::from_str(source).unwrap();
    assert_eq!(value["_version"], 2);
    for (key, locales) in value.as_object().unwrap() {
      if key == "_version" {
        continue;
      }
      assert!(
        messages.insert(key.clone(), locales.clone()).is_none(),
        "duplicate: {key}"
      );
      let en = locales["en"].as_str().unwrap();
      let zh = locales["zh"].as_str().unwrap();
      assert!(
        !en.trim().is_empty() && !zh.trim().is_empty(),
        "empty: {key}"
      );
      let placeholders = |text: &str| {
        marker
          .captures_iter(text)
          .map(|m| m[1].to_string())
          .collect::<BTreeSet<_>>()
      };
      assert_eq!(
        placeholders(en),
        placeholders(zh),
        "placeholder mismatch: {key}"
      );
    }
  }
  let original_ui_count: usize = CHUNKS[..13]
    .iter()
    .map(|source| {
      serde_json::from_str::<Value>(source)
        .unwrap()
        .as_object()
        .unwrap()
        .len()
        - 1
    })
    .sum();
  assert_eq!(original_ui_count, 595);
  assert!(messages.len() >= original_ui_count);
  assert!(messages.contains_key("Welcome back"));
  assert!(messages.contains_key("Type {path} to confirm deletion."));
}

#[test]
fn translates_original_templates_and_preserves_unknown_server_messages() {
  assert_eq!(translate("Welcome back", "zh-CN", &[]), "欢迎回来");
  assert_eq!(translate("Welcome back", "en", &[]), "Welcome back");
  assert_eq!(
    translate(
      "Type {path} to confirm deletion.",
      "en",
      &[("path", "org/project")]
    ),
    "Type org/project to confirm deletion."
  );
  assert_eq!(
    translate(
      "Type {path} to confirm deletion.",
      "zh",
      &[("path", "org/project")]
    ),
    "输入 org/project 以确认删除。"
  );
  assert_eq!(
    translate_message("zh", "invalid credentials"),
    "用户名或密码错误"
  );
  assert_eq!(
    translate_message("zh", "unknown runtime detail %{x}"),
    "unknown runtime detail %{x}"
  );
  assert_eq!(normalize_locale("fr-FR"), "en");
}

#[test]
fn explicit_locales_do_not_leak_between_concurrent_requests() {
  std::thread::scope(|scope| {
    scope.spawn(|| {
      for _ in 0..1000 {
        assert_eq!(translate_message("en", "Welcome back"), "Welcome back");
      }
    });
    scope.spawn(|| {
      for _ in 0..1000 {
        assert_eq!(translate_message("zh", "Welcome back"), "欢迎回来");
      }
    });
  });
}

#[test]
fn interpolation_preserves_literal_argument_text_and_legacy_missing_markers() {
  assert_eq!(
    translate("Type {path} to confirm deletion.", "en", &[]),
    "Type {path} to confirm deletion."
  );
  assert_eq!(
    translate(
      "Type {path} to confirm deletion.",
      "en",
      &[("path", "%{literal}")]
    ),
    "Type %{literal} to confirm deletion."
  );
}
