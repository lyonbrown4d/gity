use super::*;

#[component]
pub(super) fn VariableForm(
  context: Context,
  query: Query,
  saving: RwSignal<bool>,
  deleting: RwSignal<bool>,
) -> impl IntoView {
  let key = RwSignal::new(String::new());
  let value = RwSignal::new(String::new());
  let masked = RwSignal::new(true);
  let protected = RwSignal::new(false);
  let mut mutation = Mutation::new(query.refresh);
  mutation.busy = saving;
  let disabled =
    Signal::derive(move || !context.can_admin() || mutation.busy.get() || deleting.get());
  view! {
      <form
          class="runner-box runner-variable-form"
          on:submit=move |event| {
              event.prevent_default();
              if disabled.get_untracked() {
                  return;
              }
              let body = match model::variable_body(
                  &key.get_untracked(),
                  &value.get_untracked(),
                  masked.get_untracked(),
                  protected.get_untracked(),
              ) {
                  Ok(body) => body,
                  Err(error) => {
                      mutation.error.set(t(error));
                      return;
                  }
              };
              mutation
                  .run(
                      "PATCH",
                      format!("{}/ci/variables", context.path.get_value()),
                      body,
                      Some(
                          Callback::new(move |_| {
                              key.set(String::new());
                              value.set(String::new());
                              masked.set(true);
                              protected.set(false);
                          }),
                      ),
                  );
          }
      >
          <label>
              {t("Key")}
              <input
                  aria-label=t("Key")
                  placeholder="DEPLOY_TOKEN"
                  disabled=move || disabled.get()
                  prop:value=move || key.get()
                  on:input=move |event| key.set(event_target_value(&event).to_uppercase())
              />
          </label>
          <label>
              {t("Value")}
              <input
                  aria-label=t("Value")
                  placeholder=t("Value is hidden after save")
                  type="password"
                  autocomplete="new-password"
                  disabled=move || disabled.get()
                  prop:value=move || value.get()
                  on:input=move |event| value.set(event_target_value(&event))
              />
          </label>
          <div class="runner-actions runner-variable-buttons">
              <Toggle label="Masked" selected=masked disabled=disabled />
              <Toggle label="Protected" selected=protected disabled=disabled />
              <button
                  type="submit"
                  class="primary"
                  disabled=move || {
                      disabled.get() || key.get().trim().is_empty() || value.get().is_empty()
                  }
              >
                  <thaw::Icon icon=icondata_lu::LuPlus width="14px" height="14px" />
                  {move || t(if mutation.busy.get() { "Saving..." } else { "Save" })}
              </button>
          </div>
          <p class="runner-variable-help">
              {move || t(model::variable_guidance(masked.get(), protected.get()))}
          </p>
          <MutationError mutation=mutation />
      </form>
  }
}
#[component]
fn Toggle(label: &'static str, selected: RwSignal<bool>, disabled: Signal<bool>) -> impl IntoView {
  view! {
      <button
          type="button"
          class:secondary=move || selected.get()
          aria-pressed=move || selected.get().to_string()
          disabled=move || disabled.get()
          on:click=move |_| selected.update(|value| *value = !*value)
      >
          {t(label)}
      </button>
  }
}
