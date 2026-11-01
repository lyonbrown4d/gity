mod forms;
use crate::{api, i18n::t, ui::*};
use leptos::prelude::*;
use serde_json::Value;

#[component]
pub(super) fn Wiki(project_path: String, permissions: Query) -> impl IntoView {
  let pages = query(format!("{project_path}/wiki/pages"));
  let user = query("/users/me".into());
  let selected = RwSignal::new(String::new());
  let pending_selection = RwSignal::new(String::new());
  let composer = RwSignal::new(false);
  let mutation = Mutation::new(pages.refresh);
  let can_write = Signal::derive(move || {
    allowed(
      &permissions.data.get(),
      "project.wiki.write",
      "wiki_write",
      30,
    )
  });
  let active = Memo::new(move |_| {
    api::list(&pages.data.get())
      .into_iter()
      .find(|row| api::text(row, "slug") == selected.get())
      .unwrap_or(Value::Null)
  });
  Effect::new(move |_| {
    let rows = api::list(&pages.data.get());
    let pending = pending_selection.get();
    if !pending.is_empty() {
      if rows.iter().any(|row| api::text(row, "slug") == pending) {
        selected.set(pending);
        pending_selection.set(String::new());
      }
      return;
    }
    let slug = selected.get();
    if !rows.iter().any(|row| api::text(row, "slug") == slug) {
      let next = rows
        .first()
        .map(|row| api::text(row, "slug"))
        .unwrap_or_default();
      if next != slug {
        selected.set(next);
      }
    }
  });
  let create_path = format!("{project_path}/wiki/pages");
  view! {
      <section class="wiki-panel account-card card-enter">
          <header class="wiki-header">
              <div>
                  <h2>{t("Wiki")}</h2>
                  <p>{t("Maintain project wiki pages with markdown content.")}</p>
              </div>
              <span class="badge" class:outline=move || !can_write.get()>
                  {move || t(if can_write.get() { "Editing enabled" } else { "Read only" })}
              </span>
          </header>
          <div class="wiki-body">
              <div class="wiki-stats">
                  <WikiStat
                      label=t("Wiki pages")
                      value=Signal::derive(move || api::list(&pages.data.get()).len().to_string())
                  />
                  <WikiStat
                      label=t("Selected page")
                      value=Signal::derive(move || {
                          let title = api::text(&active.get(), "title");
                          if title.is_empty() { "--".into() } else { title }
                      })
                  />
                  <WikiStat
                      label=t("Format")
                      value=Signal::derive(move || {
                          let format = api::text(&active.get(), "format");
                          if format.is_empty() { "markdown".into() } else { format }
                      })
                  />
              </div>
              <div class="wiki-toolbar">
                  <button
                      class="primary"
                      disabled=move || !can_write.get()
                      on:click=move |_| composer.update(|open| *open = !*open)
                  >
                      {move || t(
                          if composer.get() { "Hide new wiki page form" } else { "New wiki page" },
                      )}
                  </button>
                  <button on:click=move |_| {
                      pages.refresh.update(|n| *n += 1)
                  }>{t("Reload")}</button>
              </div>
              <Show when=move || !permissions.loading.get() && !can_write.get()>
                  <aside class="wiki-notice">
                      <strong>{t("Wiki is read-only")}</strong>
                      <p>
                          {t(
                              "Your current project role can inspect wiki pages, but cannot edit them.",
                          )}
                      </p>
                  </aside>
              </Show>
              <Show when=move || !mutation.error.get().is_empty()>
                  <p class="error" role="alert">
                      {move || mutation.error.get()}
                  </p>
              </Show>
              <Show when=move || composer.get()>
                  <forms::Create
                      path=create_path.clone()
                      user=user
                      can_write=can_write
                      mutation=mutation
                      on_created=Callback::new(move |slug| {
                          pending_selection.set(slug);
                          composer.set(false);
                      })
                  />
              </Show>
              <div class="wiki-workspace">
                  <nav class="wiki-pages" aria-label=t("Wiki pages")>
                      <Show when=move || pages.loading.get()>
                          <p class="muted" role="status">
                              {t("Loading wiki pages...")}
                          </p>
                      </Show>
                      <Show when=move || !pages.error.get().is_empty()>
                          <p class="error" role="alert">
                              {move || pages.error.get()}
                          </p>
                      </Show>
                      {move || {
                          api::list(&pages.data.get())
                              .into_iter()
                              .map(|page| {
                                  let slug = api::text(&page, "slug");
                                  let target = slug.clone();
                                  let current = slug.clone();
                                  view! {
                                      <button
                                          class="wiki-page"
                                          class:selected=move || selected.get() == current
                                          on:click=move |_| selected.set(target.clone())
                                      >
                                          <strong>{api::text(&page, "title")}</strong>
                                          <small>{slug}</small>
                                          <small>
                                              {format_timestamp(&api::text(&page, "updated_at"))}
                                          </small>
                                      </button>
                                  }
                              })
                              .collect_view()
                      }}
                      <Show when=move || {
                          !pages.loading.get() && pages.error.get().is_empty()
                              && api::list(&pages.data.get()).is_empty()
                      }>
                          <WikiEmpty
                              title=t("No wiki pages found.")
                              description=Signal::derive(move || t(
                                  if can_write.get() {
                                      "Create the first wiki page to document this project."
                                  } else {
                                      "Wiki pages will appear here after maintainers create them."
                                  },
                              ))
                          />
                      </Show>
                  </nav>
                  <div class="wiki-document">
                      {move || {
                          if active.get().is_null() {
                              view! {
                                  <WikiEmpty
                                      title=t("Select or create a wiki page.")
                                      description=Signal::derive(move || t(
                                          if can_write.get() {
                                              "Create a wiki page or select one from the list to edit markdown content."
                                          } else {
                                              "Select a wiki page from the list to preview its content."
                                          },
                                      ))
                                  />
                              }
                                  .into_any()
                          } else {
                              view! {
                                  <forms::Edit
                                      page=active.get()
                                      project_path=project_path.clone()
                                      can_write=can_write
                                      mutation=mutation
                                      refresh=pages.refresh
                                      user=user
                                  />
                              }
                                  .into_any()
                          }
                      }}
                  </div>
              </div>
          </div>
      </section>
  }
}
#[component]
fn WikiStat(label: String, value: Signal<String>) -> impl IntoView {
  view! {
      <div class="wiki-stat">
          <small>{label}</small>
          <strong>{move || value.get()}</strong>
      </div>
  }
}
#[component]
fn WikiEmpty(title: String, description: Signal<String>) -> impl IntoView {
  view! {
      <div class="wiki-empty">
          <strong>{title}</strong>
          <p>{move || description.get()}</p>
      </div>
  }
}
