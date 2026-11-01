use crate::app::admin_orgs::forms::DialogClose;
use crate::{
  api,
  app::admin_orgs::model::{ProjectRow, confirmation_matches},
  i18n::t,
  ui::Mutation,
};
use leptos::prelude::*;
use serde_json::json;

#[component]
pub(super) fn CopyCloneUrl(value: String, name: String) -> impl IntoView {
  let copied = RwSignal::new(false);
  let busy = RwSignal::new(false);
  let error = RwSignal::new(false);
  view! {
      <span class="admin-project-copy">
          <button
              type="button"
              aria-label=format!("{} {name}", t("Copy Clone URL"))
              title=t("Copy Clone URL")
              disabled=move || busy.get()
              on:click=move |_| {
                  if busy.get_untracked() {
                      return;
                  }
                  busy.set(true);
                  error.set(false);
                  let value = value.clone();
                  wasm_bindgen_futures::spawn_local(async move {
                      let result = if let Some(window) = web_sys::window() {
                          wasm_bindgen_futures::JsFuture::from(
                                  window.navigator().clipboard().write_text(&value),
                              )
                              .await
                              .map(|_| ())
                      } else {
                          Err(wasm_bindgen::JsValue::NULL)
                      };
                      if busy.is_disposed() {
                          return;
                      }
                      busy.set(false);
                      copied.set(result.is_ok());
                      error.set(result.is_err());
                  });
              }
          >
              {move || {
                  view! {
                      <thaw::Icon
                          icon=if copied.get() {
                              icondata_lu::LuCheck
                          } else {
                              icondata_lu::LuCopy
                          }
                          width="14px"
                          height="14px"
                      />
                  }
              }}
          </button>
          <Show when=move || error.get()>
              <small class="error" role="alert">
                  {t("Failed to copy clone URL")}
              </small>
          </Show>
      </span>
  }
}

#[component]
pub(super) fn DeleteProjectDialog(
  open: RwSignal<bool>,
  selected: RwSignal<Option<ProjectRow>>,
  mutation: Mutation,
) -> impl IntoView {
  let verification = RwSignal::new(String::new());
  Effect::new(move |_| {
    open.get();
    verification.set(String::new());
  });
  view! {
      <thaw::Dialog open=open>
          <thaw::DialogSurface class="admin-directory-dialog">
              <DialogClose open=open />
              <thaw::DialogBody>
                  <thaw::DialogTitle>
                      {move || gity_i18n::translate(
                          "Delete project \"{name}\"?",
                          &crate::i18n::locale(),
                          &[("name", &selected.get().map(|row| row.name).unwrap_or_default())],
                      )}
                  </thaw::DialogTitle>
                  <thaw::DialogContent>
                      <p>{t("This action cannot be undone.")}</p>
                      <label
                          class="project-delete-verification"
                          for="admin-delete-project-confirmation"
                      >
                          {move || gity_i18n::translate(
                              "Type {path} to confirm deletion.",
                              &crate::i18n::locale(),
                              &[
                                  (
                                      "path",
                                      &selected.get().map(|row| row.full_path).unwrap_or_default(),
                                  ),
                              ],
                          )}
                          <input
                              id="admin-delete-project-confirmation"
                              autocomplete="off"
                              placeholder=move || {
                                  selected.get().map(|row| row.full_path).unwrap_or_default()
                              }
                              prop:value=move || verification.get()
                              on:input=move |event| verification.set(event_target_value(&event))
                          />
                      </label>
                      <Show when=move || !mutation.error.get().is_empty()>
                          <p class="error" role="alert">
                              {move || mutation.error.get()}
                          </p>
                      </Show>
                      <footer class="admin-directory-confirm-actions">
                          <button type="button" on:click=move |_| open.set(false)>
                              {t("Cancel")}
                          </button>
                          <button
                              type="button"
                              class="danger"
                              disabled=move || {
                                  mutation.busy.get()
                                      || selected
                                          .get()
                                          .is_none_or(|row| {
                                              !confirmation_matches(&row, &verification.get())
                                          })
                              }
                              on:click=move |_| {
                                  if let Some(row) = selected.get_untracked()
                                      && confirmation_matches(&row, &verification.get_untracked())
                                  {
                                      mutation
                                          .run(
                                              "DELETE",
                                              format!("/projects/{}", api::enc(&row.id)),
                                              json!({"confirmation":verification.get_untracked()}),
                                              Some(Callback::new(move |_| open.set(false))),
                                          );
                                  }
                              }
                          >
                              {move || t(
                                  if mutation.busy.get() { "Deleting..." } else { "Delete" },
                              )}
                          </button>
                      </footer>
                  </thaw::DialogContent>
              </thaw::DialogBody>
          </thaw::DialogSurface>
      </thaw::Dialog>
  }
}
