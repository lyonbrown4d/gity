use super::theme::{toggle_theme, ui_theme};
use crate::{api, i18n::t};
use leptos::prelude::*;
#[component]
pub(super) fn ViewControls() -> impl IntoView {
    let theme = expect_context::<RwSignal<thaw::Theme>>();
    let dark = RwSignal::new(api::storage("gity.theme") == "dark");
    view! {
        <div class="view-controls">
            <button
                title=t("Switch theme")
                aria-label=t("Switch theme")
                on:click=move |_| {
                    toggle_theme();
                    theme.set(ui_theme());
                    dark.set(api::storage("gity.theme") == "dark");
                }
            >
                {move || {
                    view! {
                        <thaw::Icon
                            icon=if dark.get() { icondata_lu::LuSun } else { icondata_lu::LuMoon }
                            width="16px"
                            height="16px"
                        />
                    }
                }}
            </button>
            <button
                title=t("Switch language")
                aria-label=t("Switch language")
                on:click=move |_| {
                    api::store(
                        "gity.locale",
                        if api::storage("gity.locale") == "zh" { "en" } else { "zh" },
                    );
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().reload();
                    }
                }
            >
                <thaw::Icon icon=icondata_lu::LuGlobe width="16px" height="16px" />
                {if api::storage("gity.locale") == "zh" { "中" } else { "EN" }}
            </button>
        </div>
    }
}
