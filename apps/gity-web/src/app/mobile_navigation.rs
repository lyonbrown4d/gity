use crate::{app::navigation::Navigation, i18n::t, ui::Query};
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use wasm_bindgen::{JsCast, closure::Closure};

#[component]
pub(super) fn MobileNavigationDrawer(
  path: String,
  search: String,
  me: Query,
  collapsed: RwSignal<bool>,
  open: RwSignal<bool>,
  enabled: bool,
) -> impl IntoView {
  close_on_desktop(open);
  let path = StoredValue::new(path);
  let search = StoredValue::new(search);
  view! {
      <Show when=move || enabled>
          <thaw::OverlayDrawer
              class="mobile-navigation-drawer"
              open=open
              close_on_esc=true
              mask_closeable=true
              position=thaw::DrawerPosition::Left
          >
              <button
                  class="mobile-navigation-close"
                  type="button"
                  aria-label=t("Close")
                  on:click=move |_| open.set(false)
              ></button>
              <Show when=move || open.get()>
                  <Navigation
                      path=path.get_value()
                      search=search.get_value()
                      me=me
                      collapsed=collapsed
                      drawer=true
                  />
              </Show>
          </thaw::OverlayDrawer>
      </Show>
  }
}

fn close_on_desktop(open: RwSignal<bool>) {
  let media =
    web_sys::window().and_then(|window| window.match_media("(min-width: 768px)").ok().flatten());
  if let Some(media) = media {
    let query = media.clone();
    let listener = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
      if query.matches() {
        open.set(false);
      }
    });
    let media = SendWrapper::new(media);
    let listener = SendWrapper::new(listener);
    let _ = media.add_event_listener_with_callback("change", listener.as_ref().unchecked_ref());
    on_cleanup(move || {
      let _ =
        media.remove_event_listener_with_callback("change", listener.as_ref().unchecked_ref());
      drop(listener);
    });
  }
}
