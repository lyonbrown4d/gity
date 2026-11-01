use super::model::OrganizationRow;
use crate::{api, i18n::t, ui::Mutation};
use leptos::prelude::*;
use serde_json::{Value, json};

#[component]
pub(crate) fn DialogClose(open: RwSignal<bool>) -> impl IntoView {
  view! {
      <button
          type="button"
          class="admin-directory-dialog-close"
          aria-label=t("Close")
          on:click=move |_| open.set(false)
      >
          <thaw::Icon icon=icondata_lu::LuX width="16px" height="16px" />
      </button>
  }
}

#[component]
pub(super) fn OrganizationForm(
  initial: Option<OrganizationRow>,
  mutation: Mutation,
  on_cancel: Callback<()>,
  on_saved: Callback<Value>,
) -> impl IntoView {
  let editing = initial.is_some();
  let key = RwSignal::new(
    initial
      .as_ref()
      .map(|row| row.key.clone())
      .unwrap_or_default(),
  );
  let name = RwSignal::new(
    initial
      .as_ref()
      .map(|row| row.name.clone())
      .unwrap_or_default(),
  );
  let path = initial
    .map(|row| format!("/orgs/{}", api::enc(&row.id)))
    .unwrap_or_else(|| "/orgs".into());
  let prefix = if editing { "edit-org" } else { "create-org" };
  view! {
      <form
          class="admin-directory-form"
          on:submit=move |event: leptos::ev::SubmitEvent| {
              event.prevent_default();
              mutation
                  .run_with_result(
                      if editing { "PATCH" } else { "POST" },
                      path.clone(),
                      json!(
                          {"key":key.get_untracked().trim(),"name":name.get_untracked().trim()}
                      ),
                      Some(on_saved),
                  );
          }
      >
          <label for=format!(
              "{prefix}-key",
          )>
              {t("Organization key")}
              <input
                  id=format!("{prefix}-key")
                  required
                  placeholder="acme"
                  prop:value=move || key.get()
                  on:input=move |event| key.set(event_target_value(&event))
              />
          </label>
          <label for=format!(
              "{prefix}-name",
          )>
              {t("Organization name")}
              <input
                  id=format!("{prefix}-name")
                  required
                  placeholder="Acme"
                  prop:value=move || name.get()
                  on:input=move |event| name.set(event_target_value(&event))
              />
          </label>
          <Show when=move || !mutation.error.get().is_empty()>
              <p class="error" role="alert">
                  {move || mutation.error.get()}
              </p>
          </Show>
          <footer>
              <button type="button" on:click=move |_| on_cancel.run(())>
                  {t("Cancel")}
              </button>
              <button type="submit" class="primary" disabled=move || mutation.busy.get()>
                  {move || t(
                      if editing {
                          if mutation.busy.get() { "Saving..." } else { "Save" }
                      } else if mutation.busy.get() {
                          "Creating..."
                      } else {
                          "Create"
                      },
                  )}
              </button>
          </footer>
      </form>
  }
}

#[component]
pub(super) fn MemberForm(
  organization_id: String,
  mutation: Mutation,
  on_cancel: Callback<()>,
  on_saved: Callback<()>,
) -> impl IntoView {
  let user = RwSignal::new(String::new());
  let role = RwSignal::new("developer".to_string());
  let path = format!("/orgs/{}/members", api::enc(&organization_id));
  view! {
      <form
          class="admin-directory-form"
          on:submit=move |event: leptos::ev::SubmitEvent| {
              event.prevent_default();
              mutation
                  .run(
                      "POST",
                      path.clone(),
                      json!({"user_id":user.get_untracked().trim(),"role":role.get_untracked()}),
                      Some(on_saved),
                  );
          }
      >
          <label for="member-user-id">
              {t("User ID")}
              <input
                  id="member-user-id"
                  required
                  placeholder="user-id"
                  inputmode="numeric"
                  prop:value=move || user.get()
                  on:input=move |event| user.set(event_target_value(&event))
              />
          </label>
          <label for="member-role">
              {t("Role")}
              <select
                  id="member-role"
                  aria-label=t("Role")
                  prop:value=move || role.get()
                  on:change=move |event| role.set(event_target_value(&event))
              >
                  <option value="developer">{t("member")}</option>
                  <option value="owner">{t("owner")}</option>
              </select>
          </label>
          <Show when=move || !mutation.error.get().is_empty()>
              <p class="error" role="alert">
                  {move || mutation.error.get()}
              </p>
          </Show>
          <footer>
              <button type="button" on:click=move |_| on_cancel.run(())>
                  {t("Cancel")}
              </button>
              <button type="submit" class="primary" disabled=move || mutation.busy.get()>
                  {move || t(if mutation.busy.get() { "Adding..." } else { "Add Member" })}
              </button>
          </footer>
      </form>
  }
}
