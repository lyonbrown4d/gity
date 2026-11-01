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
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = app::App;
    }
}
