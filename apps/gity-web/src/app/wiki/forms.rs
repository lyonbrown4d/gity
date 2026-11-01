use super::*;
use crate::markdown::Markdown;
use serde_json::json;

#[component]
pub(super) fn Create(
  path: String,
  user: Query,
  can_write: Signal<bool>,
  mutation: Mutation,
  on_created: Callback<String>,
) -> impl IntoView {
  let title = RwSignal::new(String::new());
  let slug = RwSignal::new(String::new());
  let content = RwSignal::new(String::new());
  let submit = move |event: leptos::ev::SubmitEvent| {
    event.prevent_default();
    if !can_write.get_untracked() {
      return;
    }
    if title.get_untracked().trim().is_empty() {
      mutation.error.set(t("Wiki page title is required"));
      return;
    }
    mutation.run_with_result("POST",path.clone(),json!({"title":title.get_untracked().trim(),"slug":slug.get_untracked().trim(),"content":content.get_untracked(),"format":"markdown"}),Some(Callback::new(move |page: Value| on_created.run(api::text(&page, "slug")))));
  };
  view! {
      <form class="wiki-composer" on:submit=submit>
          <div class="wiki-create-fields">
              <label for="wiki-title">
                  {t("Wiki page title")}
                  <input
                      id="wiki-title"
                      required
                      placeholder=t("Getting Started")
                      prop:value=move || title.get()
                      on:input=move |event| title.set(event_target_value(&event))
                  />
              </label>
              <label for="wiki-slug">
                  {t("Slug optional")}
                  <input
                      id="wiki-slug"
                      placeholder="getting-started"
                      prop:value=move || slug.get()
                      on:input=move |event| slug.set(event_target_value(&event))
                  />
              </label>
              <label for="wiki-author-id">
                  {t("Author user ID")}
                  <input
                      id="wiki-author-id"
                      readonly
                      prop:value=move || api::text(&user.data.get(), "id")
                  />
              </label>
          </div>
          <label for="wiki-content">
              {t("Content")}
              <textarea
                  id="wiki-content"
                  class="wiki-new-content"
                  placeholder="# Getting Started"
                  prop:value=move || content.get()
                  on:input=move |event| content.set(event_target_value(&event))
              />
          </label>
          <div class="wiki-submit">
              <button class="primary" disabled=move || !can_write.get() || mutation.busy.get()>
                  {move || t(
                      if mutation.busy.get() {
                          "Creating wiki page..."
                      } else {
                          "Create wiki page"
                      },
                  )}
              </button>
          </div>
      </form>
  }
}

#[component]
pub(super) fn Edit(
  page: Value,
  project_path: String,
  can_write: Signal<bool>,
  mutation: Mutation,
  refresh: RwSignal<u64>,
  user: Query,
) -> impl IntoView {
  let title = RwSignal::new(api::text(&page, "title"));
  let content = RwSignal::new(api::text(&page, "content"));
  let slug = api::text(&page, "slug");
  let format = api::text(&page, "format");
  let editor = api::text(&page, "last_edited_by_user_id");
  let path = format!("{project_path}/wiki/pages/{}", api::enc(&slug));
  let delete_path = path.clone();
  let confirmation = gity_i18n::translate(
    "Delete wiki page \"{title}\"?",
    &crate::i18n::locale(),
    &[("title", &api::text(&page, "title"))],
  );
  let submit = move |event: leptos::ev::SubmitEvent| {
    event.prevent_default();
    if !can_write.get_untracked() {
      return;
    }
    if title.get_untracked().trim().is_empty() {
      mutation.error.set(t("Wiki page title is required"));
      return;
    }
    mutation.run(
      "PATCH",
      path.clone(),
      json!({"title":title.get_untracked().trim(),"content":content.get_untracked()}),
      None,
    );
  };
  view! {
      <form class="wiki-edit" on:submit=submit>
          <div class="wiki-document-header">
              <div>
                  <div class="wiki-tags">
                      <span class="badge">{slug}</span>
                      <span class="badge outline">{format}</span>
                  </div>
                  <small>{t("Last edited by")}": "{editor.clone()}</small>
              </div>
              <Action
                  label=t("Delete")
                  method="DELETE".into()
                  path=delete_path
                  refresh=refresh
                  confirm=true
                  disabled=Signal::derive(move || !can_write.get() || mutation.busy.get())
                  confirmation_title=confirmation
                  confirmation_description=t("This action cannot be undone.")
              />
          </div>
          <div class="wiki-edit-fields">
              <label for="wiki-draft-title">
                  {t("Wiki page title")}
                  <input
                      id="wiki-draft-title"
                      required
                      disabled=move || !can_write.get()
                      prop:value=move || title.get()
                      on:input=move |event| title.set(event_target_value(&event))
                  />
              </label>
              <label for="wiki-editor-id">
                  {t("Editor user ID")}
                  <input
                      id="wiki-editor-id"
                      readonly
                      prop:value=move || api::text(&user.data.get(), "id")
                  />
              </label>
          </div>
          <div class="wiki-editor-preview">
              <label for="wiki-draft-content">
                  {t("Wiki page content")}
                  <textarea
                      id="wiki-draft-content"
                      disabled=move || !can_write.get()
                      prop:value=move || content.get()
                      on:input=move |event| content.set(event_target_value(&event))
                  />
              </label>
              <section class="wiki-preview">
                  <strong>{t("Preview")}</strong>
                  <article class="markdown-body">
                      {move || {
                          if content.get().trim().is_empty() {
                              view! { <p>{t("Empty wiki page.")}</p> }.into_any()
                          } else {
                              view! { <Markdown content=content.get() /> }.into_any()
                          }
                      }}
                  </article>
              </section>
          </div>
          <div class="wiki-submit">
              <button class="primary" disabled=move || !can_write.get() || mutation.busy.get()>
                  {move || t(
                      if mutation.busy.get() { "Saving wiki page..." } else { "Save wiki page" },
                  )}
              </button>
          </div>
      </form>
  }
}
