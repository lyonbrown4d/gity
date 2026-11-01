use crate::{api, ui::*};
use leptos::prelude::*;
use leptos_router::components::A;
#[component]
pub(super) fn QuickJump() -> impl IntoView {
  let open = RwSignal::new(false);
  let term = RwSignal::new(String::new());
  let projects = query("/projects".into());
  let handle = window_event_listener(leptos::ev::keydown, move |event| {
    if (event.ctrl_key() || event.meta_key()) && event.key().to_lowercase() == "k" {
      event.prevent_default();
      open.set(!open.get_untracked());
    }
  });
  on_cleanup(move || handle.remove());
  view! {
      <button class="quick-jump" on:click=move |_| open.set(true)>
          <span>
              <thaw::Icon icon=icondata_lu::LuSearch width="16px" height="16px" />
              {crate::i18n::t("Quick jump")}
          </span>
          <kbd>{crate::i18n::t("Ctrl K")}</kbd>
      </button>
      <thaw::Dialog open=open>
          <thaw::DialogSurface>
              <thaw::DialogBody>
                  <thaw::DialogTitle>{crate::i18n::t("Quick jump")}</thaw::DialogTitle>
                  <thaw::DialogContent>
                      <input
                          class="quick-search"
                          type="search"
                          placeholder="Search projects or pages…"
                          aria-label="Search projects or pages"
                          on:input=move |e| term.set(event_target_value(&e))
                      />
                      <nav class="quick-results">
                          {move || {
                              [
                                  ("Dashboard", "/app/dashboard"),
                                  ("Projects", "/app/projects"),
                                  ("Inbox", "/app/todos"),
                                  ("Profile", "/app/profile"),
                              ]
                                  .into_iter()
                                  .filter(|(label, _)| {
                                      label.to_lowercase().contains(&term.get().to_lowercase())
                                  })
                                  .map(|(label, url)| {
                                      view! {
                                          <A href={url} {..} on:click=move |_| open.set(false)>
                                              {label}
                                          </A>
                                      }
                                  })
                                  .collect_view()
                          }}
                          {move || {
                              api::list(&projects.data.get())
                                  .into_iter()
                                  .filter(|p| {
                                      api::text(p, "name")
                                          .to_lowercase()
                                          .contains(&term.get().to_lowercase())
                                  })
                                  .take(12)
                                  .map(|p| {
                                      view! {
                                          <A
                                              href=format!(
                                                  "/app/projects/{}/{}",
                                                  api::text(&p, "organization_id"),
                                                  api::text(&p, "id"),
                                              )
                                              {..}
                                              on:click=move |_| open.set(false)
                                          >
                                              {api::text(&p, "name")}
                                          </A>
                                      }
                                  })
                                  .collect_view()
                          }}
                      </nav>
                  </thaw::DialogContent>
              </thaw::DialogBody>
          </thaw::DialogSurface>
      </thaw::Dialog>
  }
}
