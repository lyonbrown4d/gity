use crate::api;
pub(super) fn ui_theme() -> thaw::Theme {
  let mut theme = if api::storage("gity.theme") == "dark" {
    thaw::Theme::dark()
  } else {
    thaw::Theme::light()
  };
  theme.common.set_font_family_base(
    "\"Aptos\",\"Segoe UI Variable\",\"IBM Plex Sans\",ui-sans-serif,system-ui,sans-serif".into(),
  );
  theme.common.set_border_radius_medium("6px".into());
  theme
    .color
    .set_color_neutral_foreground_on_brand("hsl(var(--primary-foreground))".into());
  theme
    .color
    .set_color_brand_background("hsl(var(--primary))".into());
  theme
    .color
    .set_color_brand_background_hover("hsl(var(--primary) / .9)".into());
  theme
    .color
    .set_color_brand_background_pressed("hsl(var(--primary) / .8)".into());
  theme
    .color
    .set_color_neutral_background_1("hsl(var(--card))".into());
  theme
    .color
    .set_color_neutral_foreground_1("hsl(var(--foreground))".into());
  theme
}
pub fn apply_theme() {
  let stored = api::storage("gity.theme");
  let dark = if stored.is_empty() {
    web_sys::window()
      .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok().flatten())
      .is_some_and(|m| m.matches())
  } else {
    stored == "dark"
  };
  let mode = if dark { "dark" } else { "light" };
  api::store("gity.theme", mode);
  if let Some(el) = web_sys::window()
    .and_then(|w| w.document())
    .and_then(|d| d.document_element())
  {
    let _ = el.set_attribute("data-theme", mode);
    let _ = el.set_attribute("class", if dark { "dark" } else { "" });
    let _ = el.set_attribute("style", &format!("color-scheme: {mode}"));
  }
  crate::editors::set_theme(dark);
}
pub(super) fn toggle_theme() {
  let mode = if api::storage("gity.theme") == "dark" {
    "light"
  } else {
    "dark"
  };
  api::store("gity.theme", mode);
  apply_theme();
}
