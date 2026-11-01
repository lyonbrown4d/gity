use super::*;
const COMMAND: &str = "git lfs install\ngit lfs track \"*.psd\"\ngit add .gitattributes\ngit commit -m \"Track large assets with Git LFS\"\ngit push";
#[component]
pub(super) fn Setup() -> impl IntoView {
  let copied = RwSignal::new(false);
  let busy = RwSignal::new(false);
  let error = RwSignal::new(String::new());
  let copy = move |_| {
    if busy.get_untracked() {
      return;
    }
    busy.set(true);
    error.set(String::new());
    wasm_bindgen_futures::spawn_local(async move {
      let result = if let Some(window) = web_sys::window() {
        wasm_bindgen_futures::JsFuture::from(window.navigator().clipboard().write_text(COMMAND))
          .await
          .map(|_| ())
      } else {
        Err(wasm_bindgen::JsValue::NULL)
      };
      if busy.is_disposed() {
        return;
      }
      busy.set(false);
      if result.is_err() {
        error.set(t("Failed to copy setup command"));
        return;
      }
      copied.set(true);
      leptos::leptos_dom::helpers::set_timeout(
        move || {
          if !copied.is_disposed() {
            copied.set(false);
          }
        },
        std::time::Duration::from_millis(1500),
      );
    });
  };
  view! {
      <aside class="lfs-setup">
          <header>
              <strong>{t("Client setup")}</strong>
              <button type="button" on:click=copy disabled=move || busy.get()>
                  <thaw::Icon icon=icondata_lu::LuCopy width="16px" height="16px" />
                  {move || t(if copied.get() { "Copied" } else { "Copy" })}
              </button>
          </header>
          <p>{t("Use standard Git LFS clients against the project clone URL.")}</p>
          <pre>{COMMAND}</pre>
          <Show when=move || !error.get().is_empty()>
              <p class="error" role="alert">
                  {move || error.get()}
              </p>
          </Show>
      </aside>
  }
}
