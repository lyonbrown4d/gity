mod api;
mod app;
mod catalog;
mod editors;
mod files;
mod i18n;
mod issues;
mod markdown;
mod source;
mod ui;
fn main() {
  #[cfg(target_arch = "wasm32")]
  {
    app::apply_theme();
    i18n::apply_locale();
    leptos::mount::mount_to_body(app::App);
    // Drop the static boot skeleton once the application has rendered.
    if let Some(boot) = web_sys::window()
      .and_then(|window| window.document())
      .and_then(|document| document.get_element_by_id("boot"))
    {
      boot.remove();
    }
  }
  #[cfg(not(target_arch = "wasm32"))]
  {
    let _ = app::App;
  }
}
