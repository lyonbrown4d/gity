use super::{Context, SectionHeader};
use crate::{api, i18n::t, ui::*};
use leptos::prelude::*;
use serde_json::json;

#[component]
pub(super) fn Keys(context: Context) -> impl IntoView {
  let title = RwSignal::new(String::new());
  let public = RwSignal::new(String::new());
  let push = RwSignal::new(false);
  let submit = move |e: leptos::ev::SubmitEvent| {
    e.prevent_default();
    if context.disabled.get_untracked() {
      return;
    }
    if title.get_untracked().trim().is_empty() || public.get_untracked().trim().is_empty() {
      context
        .mutation
        .error
        .set(t("Deploy key title and public key are required."));
      return;
    }
    context.mutation.run("POST",format!("{}/deploy-keys",context.path.get_value()),json!({"title":title.get_untracked(),"public_key":public.get_untracked(),"can_push":push.get_untracked()}),Some(Callback::new(move|()|{title.set(String::new());public.set(String::new());push.set(false);})))
  };
  view! {
      <section class="settings-credential-section">
          <SectionHeader
              title="Deploy keys"
              description="Use SSH public keys for project-scoped clone, pull and optional push."
              query=context.keys
          />
          <form class="settings-key-form" on:submit=submit>
              <label for="deploy-key-title">
                  {t("Title")}
                  <input
                      id="deploy-key-title"
                      placeholder=t("Build server")
                      prop:value=move || title.get()
                      on:input=move |e| title.set(event_target_value(&e))
                      disabled=move || context.disabled.get()
                  />
              </label>
              <label for="deploy-key-public-key">
                  {t("Public key")}
                  <textarea
                      id="deploy-key-public-key"
                      placeholder="ssh-ed25519 AAAA..."
                      prop:value=move || public.get()
                      on:input=move |e| public.set(event_target_value(&e))
                      disabled=move || context.disabled.get()
                  />
              </label>
              <div class="settings-inline">
                  <button
                      type="button"
                      class:selected=move || push.get()
                      aria-pressed=move || push.get().to_string()
                      disabled=move || context.disabled.get()
                      on:click=move |_| push.update(|v| *v = !*v)
                  >
                      {t("Can push")}
                  </button>
                  <button
                      type="submit"
                      class="primary"
                      disabled=move || {
                          context.disabled.get() || title.get().trim().is_empty()
                              || public.get().trim().is_empty()
                      }
                  >
                      {move || t(
                          if context.mutation.busy.get() { "Adding..." } else { "Add key" },
                      )}
                  </button>
              </div>
          </form>
          <Status q=context.keys />
          <div class="settings-credential-list">
              <Show when=move || {
                  !context.keys.loading.get() && api::list(&context.keys.data.get()).is_empty()
              }>
                  <p>{t("No deploy keys yet.")}</p>
              </Show>
              {move || {
                  api::list(&context.keys.data.get())
                      .into_iter()
                      .map(|key| {
                          view! {
                              <div class="settings-credential-row">
                                  <div>
                                      <div class="settings-inline">
                                          <strong>{api::text(&key, "title")}</strong>
                                          <span class="badge" class:outline=key["can_push"] != true>
                                              {t(
                                                  if key["can_push"] == true {
                                                      "can push"
                                                  } else {
                                                      "read only"
                                                  },
                                              )}
                                          </span>
                                      </div>
                                      <small>{api::text(&key, "fingerprint")}</small>
                                      <code>{api::text(&key, "public_key")}</code>
                                  </div>
                                  <Action
                                      label="Delete".into()
                                      path=format!(
                                          "{}/deploy-keys/{}",
                                          context.path.get_value(),
                                          api::enc(&api::text(&key, "id")),
                                      )
                                      method="DELETE".into()
                                      refresh=context.keys.refresh
                                      busy_state=context.mutation.busy
                                      disabled=context.disabled
                                      confirm=true
                                      confirmation_title=t("Delete deploy key?")
                                      confirmation_description=t(
                                          "Deployments using this key will no longer be trusted by this project.",
                                      )
                                      confirmation_label=t("Delete")
                                  />
                              </div>
                          }
                      })
                      .collect_view()
              }}
          </div>
      </section>
  }
}
