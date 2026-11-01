//! Browser locale preference over the shared frontend/backend catalog.
pub fn locale() -> String {
  gity_i18n::normalize_locale(&crate::api::storage("gity.locale")).to_string()
}
pub fn t(text: &str) -> String {
  gity_i18n::translate(text, &locale(), &[])
}
#[cfg(target_arch = "wasm32")]
pub fn apply_locale() {
  if let Some(el) = web_sys::window()
    .and_then(|w| w.document())
    .and_then(|d| d.document_element())
  {
    let _ = el.set_attribute(
      "lang",
      if crate::api::storage("gity.locale") == "zh" {
        "zh-CN"
      } else {
        "en"
      },
    );
  }
}
