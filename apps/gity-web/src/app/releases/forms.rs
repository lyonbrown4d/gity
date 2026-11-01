use super::*;
use serde_json::json;

#[component]
pub(super) fn ReleaseForm(context: Context) -> impl IntoView {
  let editing = context.editing.get_untracked();
  let id = api::text(&editing, "id");
  let is_editing = !id.is_empty();
  let tag = RwSignal::new(api::text(&editing, "tag_name"));
  let source = RwSignal::new(String::new());
  let name = RwSignal::new(api::text(&editing, "name"));
  let description = RwSignal::new(api::text(&editing, "description"));
  let create_tag = RwSignal::new(true);
  let mutation = Mutation::new(context.tags.refresh);
  Effect::new(move |_| {
    let default = api::text(&context.project.data.get(), "default_branch");
    if source.get_untracked().is_empty() {
      source.set(if default.is_empty() {
        "main".into()
      } else {
        default
      });
    }
  });
  view! {
      <form
          class="releases-box releases-stack releases-form"
          on:submit=move |e| {
              e.prevent_default();
              if !context.can_admin() {
                  return;
              }
              if tag.get_untracked().trim().is_empty() {
                  mutation.error.set(t("Tag name is required."));
                  return;
              }
              let path = format!(
                  "{}/releases{}",
                  context.path.get_value(),
                  if is_editing { format!("/{id}") } else { String::new() },
              );
              let normalized = model::release_body(
                  &tag.get_untracked(),
                  &name.get_untracked(),
                  &description.get_untracked(),
                  &source.get_untracked(),
                  &api::text(&context.project.data.get_untracked(), "default_branch"),
                  create_tag.get_untracked(),
              );
              let body = if is_editing {
                  json!({"name":normalized["name"], "description":normalized["description"]})
              } else {
                  normalized
              };
              mutation
                  .run_with_result(
                      if is_editing { "PATCH" } else { "POST" },
                      path,
                      body,
                      Some(
                          Callback::new(move |value: Value| {
                              context.pending.set(api::text(&value["release"], "id"));
                              context.composer.set(false);
                              context.editing.set(Value::Null);
                          }),
                      ),
                  );
          }
      >
          <div class="release-create-fields">
              <label>
                  {t("Tag name")}
                  <input
                      aria-label=t("Tag name")
                      placeholder="v0.1.0"
                      disabled=is_editing
                      prop:value=move || tag.get()
                      on:input=move |e| tag.set(event_target_value(&e))
                      required
                  />
              </label>
              <label>
                  {t("Source ref")}
                  <input
                      aria-label=t("Source ref")
                      placeholder="main"
                      disabled=is_editing
                      prop:value=move || source.get()
                      on:input=move |e| source.set(event_target_value(&e))
                  />
              </label>
              <label>
                  {t("Create tag")}
                  <select
                      aria-label=t("Create tag")
                      disabled=is_editing
                      prop:value=move || if create_tag.get() { "yes" } else { "no" }
                      on:change=move |e| create_tag.set(event_target_value(&e) == "yes")
                  >
                      <option value="yes">{t("Yes")}</option>
                      <option value="no">{t("No")}</option>
                  </select>
              </label>
          </div>
          <label>
              {t("Release name")}
              <input
                  aria-label=t("Release name")
                  placeholder="Gity 0.1.0 beta"
                  prop:value=move || name.get()
                  on:input=move |e| name.set(event_target_value(&e))
              />
          </label>
          <label>
              {t("Release notes")}
              <textarea
                  aria-label=t("Release notes")
                  placeholder=t(
                      "Describe user-facing changes, upgrade notes, and known limitations.",
                  )
                  rows="4"
                  prop:value=move || description.get()
                  on:input=move |e| description.set(event_target_value(&e))
              ></textarea>
          </label>
          <Show when=move || !mutation.error.get().is_empty()>
              <p class="error" role="alert">
                  {move || mutation.error.get()}
              </p>
          </Show>
          <div class="releases-end">
              <button
                  type="submit"
                  class="primary"
                  disabled=move || {
                      !context.can_admin() || mutation.busy.get() || tag.get().trim().is_empty()
                  }
              >
                  {move || t(
                      if mutation.busy.get() {
                          "Saving release..."
                      } else if is_editing {
                          "Update release"
                      } else {
                          "Create release"
                      },
                  )}
              </button>
          </div>
      </form>
  }
}
