//! Personal SSH credentials share the account settings visual and mutation primitives.
use crate::{api, i18n::t, ui::*};
use leptos::prelude::*;
use serde_json::json;

#[component]
pub(crate) fn PersonalSshKeys() -> impl IntoView {
  let keys = query("/users/me/ssh-keys".into());
  let mutation = Mutation::new(keys.refresh);
  let title = RwSignal::new(String::new());
  let public = RwSignal::new(String::new());
  let disabled = Signal::derive(move || keys.loading.get() || mutation.busy.get());
  let submit = move |event: leptos::ev::SubmitEvent| {
    event.prevent_default();
    if disabled.get_untracked() {
      return;
    }
    mutation.run(
      "POST",
      "/users/me/ssh-keys".into(),
      json!({"title":title.get_untracked(), "public_key":public.get_untracked()}),
      Some(Callback::new(move |()| {
        title.set(String::new());
        public.set(String::new());
      })),
    );
  };
  view! {
    <section class="settings-card account-card card-enter">
      <header>
        <h2>{t("Personal SSH keys")}</h2>
        <p>{t("Use your account permissions to clone, pull and push over SSH as git.")}</p>
      </header>
      <div class="settings-body settings-credentials">
        <form class="settings-key-form" on:submit=submit>
          <label for="personal-ssh-title">
            {t("Title")}
            <input id="personal-ssh-title" maxlength="255" required
              prop:value=move || title.get()
              on:input=move |event| title.set(event_target_value(&event))
              disabled=move || disabled.get() />
          </label>
          <label for="personal-ssh-public-key">
            {t("Public key")}
            <textarea id="personal-ssh-public-key" placeholder="ssh-ed25519 AAAA..." maxlength="16384" required
              prop:value=move || public.get()
              on:input=move |event| public.set(event_target_value(&event))
              disabled=move || disabled.get() />
          </label>
          <div class="settings-inline">
            <p>{t("Paste a public key only. Keep your private key on your device.")}</p>
            <button type="submit" class="primary"
              disabled=move || disabled.get() || title.get().trim().is_empty() || public.get().trim().is_empty()>
              {move || t(if mutation.busy.get() { "Adding..." } else { "Add key" })}
            </button>
          </div>
        </form>
        <Show when=move || !mutation.error.get().is_empty()>
          <p class="error" role="alert">{move || mutation.error.get()}</p>
        </Show>
        <Status q=keys />
        <Show when=move || !keys.loading.get() && keys.error.get().is_empty() && api::list(&keys.data.get()).is_empty()>
          <p>{t("No personal SSH keys yet.")}</p>
        </Show>
        <div class="settings-credential-list">
          {move || api::list(&keys.data.get()).into_iter().map(|key| {
            let last_used = api::text(&key, "last_used_at");
            let last_used = if last_used.is_empty() { t("Never") } else { format_timestamp(&last_used) };
            view! {
              <div class="settings-credential-row">
                <div>
                  <strong>{api::text(&key, "title")}</strong>
                  <small>{api::text(&key, "fingerprint")}</small>
                  <code>{api::text(&key, "public_key")}</code>
                  <small>{t("Created")}{": "}{format_timestamp(&api::text(&key, "created_at"))}</small>
                  <small>{t("Last used")}{": "}{last_used}</small>
                </div>
                <Action label="Delete".into() method="DELETE".into()
                  path=format!("/users/me/ssh-keys/{}", api::enc(&api::text(&key,"id")))
                  refresh=keys.refresh busy_state=mutation.busy disabled=disabled confirm=true
                  confirmation_title=t("Delete personal SSH key?")
                  confirmation_description=t("This key will no longer authenticate new Git commands for your account.")
                  confirmation_label=t("Delete") />
              </div>
            }
          }).collect_view()}
        </div>
      </div>
    </section>
  }
}
