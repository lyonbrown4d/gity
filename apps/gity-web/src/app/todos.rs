use crate::{api, i18n::t, ui::query};
use leptos::prelude::*;
use serde_json::{Value, json};

mod presentation;
use presentation::{action_url, format_kind, format_time};

#[component]
pub(super) fn Todos(search: String) -> impl IntoView {
  let params = web_sys::UrlSearchParams::new_with_str(&search).ok();
  let done = params.as_ref().and_then(|p| p.get("state")).as_deref() == Some("done");
  let state = if done { "done" } else { "pending" };
  let q = query(format!("/todos?state={state}&limit=100"));
  let active = RwSignal::new(String::new());
  let error = RwSignal::new(String::new());
  let mark_done = Callback::new(move |id: String| {
    if !active.get_untracked().is_empty() {
      return;
    }
    active.set(id.clone());
    error.set(String::new());
    wasm_bindgen_futures::spawn_local(async move {
      let result = api::request(
        "POST",
        &format!("/todos/{}/done", api::enc(&id)),
        Some(json!({})),
      )
      .await;
      if active.is_disposed() {
        return;
      }
      match result {
        Ok(_) => q.refresh.update(|n| *n += 1),
        Err(cause) => error.set(cause),
      }
      active.set(String::new());
    });
  });
  let state_url = move |state: &str| {
    if let Some(params) = &params {
      params.set("state", state);
      format!("/app/todos?{}", params.to_string())
    } else {
      format!("/app/todos?state={state}")
    }
  };
  view! {
      <div class="inbox-page page-enter">
          <section class="inbox-hero">
              <div>
                  <p class="product-eyebrow">
                      <span></span>
                      {t("Personal queue")}
                  </p>
                  <h1>{t("Inbox")}</h1>
                  <p class="inbox-description">
                      {t(
                          "Review project activity that needs your attention, then clear it from the queue.",
                      )}
                  </p>
              </div>
              <aside>
                  <p class="eyebrow">{t(if done { "Done" } else { "Pending" })}</p>
                  <strong>
                      {move || {
                          if q.loading.get() {
                              "--".into()
                          } else {
                              api::list(&q.data.get()).len().to_string()
                          }
                      }}
                  </strong>
                  <small>{t("Items in this view")}</small>
              </aside>
          </section>
          <div class="inbox-controls">
              <nav class="inbox-tabs" aria-label=t("Todo state")>
                  <a
                      href=state_url("pending")
                      class:active=!done
                      aria-current=if done { None } else { Some("page") }
                  >
                      {t("Pending")}
                  </a>
                  <a
                      href=state_url("done")
                      class:active=done
                      aria-current=if done { Some("page") } else { None }
                  >
                      {t("Done")}
                  </a>
              </nav>
              <button
                  disabled=move || q.loading.get()
                  on:click=move |_| q.refresh.update(|n| *n += 1)
              >
                  {move || t(if q.loading.get() { "Refreshing..." } else { "Refresh" })}
              </button>
          </div>
          <Show when=move || !q.error.get().is_empty()>
              <div class="error inbox-error" role="alert">
                  <span>{move || q.error.get()}</span>
                  <button on:click=move |_| {
                      q.refresh.update(|n| *n += 1)
                  }>{t("Try again")}</button>
              </div>
          </Show>
          <Show when=move || !error.get().is_empty()>
              <p class="error" role="alert">
                  {move || error.get()}
              </p>
          </Show>
          <Show when=move || q.loading.get()>
              <div class="inbox-loading" aria-label="Loading todos">
                  {(0..3)
                      .map(|_| view! { <div class="loading-placeholder"></div> })
                      .collect_view()}
              </div>
          </Show>
          <Show when=move || !q.loading.get() && q.error.get().is_empty()>
              <Show when=move || api::list(&q.data.get()).is_empty()>
                  <div class="inbox-empty">
                      <span>
                          <thaw::Icon
                              icon=icondata_lu::LuCircleCheckBig
                              width="24px"
                              height="24px"
                          />
                      </span>
                      <h2>
                          {t(
                              if done {
                                  "No completed todos yet."
                              } else {
                                  "You're all caught up."
                              },
                          )}
                      </h2>
                      <p>
                          {t(
                              if done {
                                  "Completed items remain available for reference."
                              } else {
                                  "New project activity will appear here."
                              },
                          )}
                      </p>
                  </div>
              </Show>
              <div class="inbox-cards">
                  {move || {
                      api::list(&q.data.get())
                          .into_iter()
                          .map(|todo| {
                              view! { <TodoCard todo=todo active=active mark_done=mark_done /> }
                          })
                          .collect_view()
                  }}
              </div>
          </Show>
      </div>
  }
}

#[component]
fn TodoCard(todo: Value, active: RwSignal<String>, mark_done: Callback<String>) -> impl IntoView {
  let done = api::text(&todo, "state") == "done";
  let id = api::text(&todo, "id");
  let active_id = id.clone();
  let label_id = id.clone();
  let title = api::text(&todo, "title");
  view! {
      <article class="account-card todo-card card-enter" aria-label=title.clone()>
          <div class="todo-heading">
              <div>
                  <span class="todo-icon">
                      <thaw::Icon
                          icon=if done {
                              icondata_lu::LuCircleCheckBig
                          } else {
                              icondata_lu::LuInbox
                          }
                          width="20px"
                          height="20px"
                      />
                  </span>
                  <div>
                      <h2>{title.clone()}</h2>
                      <p>{api::text(&todo, "summary")}</p>
                  </div>
              </div>
              <span class="badge" class:primary=!done>
                  {t(if done { "done" } else { "pending" })}
              </span>
          </div>
          <footer>
              <div class="todo-meta">
                  <span class="badge outline">{format_kind(&api::text(&todo, "kind"))}</span>
                  <time datetime=api::text(
                      &todo,
                      "created_at",
                  )>{format_time(&api::text(&todo, "created_at"))}</time>
              </div>
              <div class="todo-actions">
                  <a class="button" href=action_url(&todo)>
                      {t("Open")}
                      <thaw::Icon icon=icondata_lu::LuArrowUpRight width="16px" height="16px" />
                  </a>
                  {if done {
                      None
                  } else {
                      Some(
                          view! {
                              <button
                                  class="primary"
                                  disabled=move || active.get() == active_id
                                  on:click=move |_| mark_done.run(id.clone())
                              >
                                  <thaw::Icon
                                      icon=icondata_lu::LuCheck
                                      width="16px"
                                      height="16px"
                                  />
                                  {move || t(
                                      if active.get() == label_id {
                                          "Marking done..."
                                      } else {
                                          "Mark done"
                                      },
                                  )}
                              </button>
                          },
                      )
                  }}
              </div>
          </footer>
      </article>
  }
}
