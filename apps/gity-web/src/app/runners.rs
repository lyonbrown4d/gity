mod guide;
mod list;
mod model;
mod register;
mod variable_form;
mod variables;
use crate::{api, i18n::t, ui::*};
use leptos::prelude::*;
use serde_json::Value;

#[derive(Clone, Copy)]
struct Context {
  path: StoredValue<String>,
  rows: Query,
  permissions: Query,
  composer: RwSignal<bool>,
  token: RwSignal<String>,
  deleting: RwSignal<bool>,
  draft_name: RwSignal<String>,
  draft_description: RwSignal<String>,
  draft_tags: RwSignal<String>,
}
impl Context {
  fn can_admin(self) -> bool {
    allowed(
      &self.permissions.data.get(),
      "project.runners.admin",
      "runner_admin",
      40,
    )
  }
}

#[component]
pub(super) fn Runners(project_path: String, permissions: Query) -> impl IntoView {
  let context = Context {
    path: StoredValue::new(project_path.clone()),
    rows: query(format!("{project_path}/runners")),
    permissions,
    composer: RwSignal::new(false),
    token: RwSignal::new(String::new()),
    deleting: RwSignal::new(false),
    draft_name: RwSignal::new(String::new()),
    draft_description: RwSignal::new(String::new()),
    draft_tags: RwSignal::new("linux,go".to_string()),
  };
  let runners = Memo::new(move |_| {
    api::list(&context.rows.data.get())
      .iter()
      .map(model::Runner::from)
      .collect::<Vec<_>>()
  });
  let stats = Memo::new(move |_| model::Stats::from_runners(&runners.get()));
  view! {
      <section class="runners-panel account-card card-enter">
          <header class="runners-header">
              <h2>{t("Runners")}</h2>
              <p>{t("Register external runners and inspect project runner health.")}</p>
          </header>
          <div class="runners-body">
              <div class="runner-stats">
                  <Stat
                      label="Registered runners"
                      value=Signal::derive(move || stats.get().registered)
                      description=Signal::derive(move || t(
                          if stats.get().registered > 0 {
                              "Available to this project"
                          } else {
                              "No capacity yet"
                          },
                      ))
                  />
                  <Stat
                      label="Online runners"
                      value=Signal::derive(move || stats.get().online)
                      description=Signal::stored(t("Heartbeat received recently"))
                      active=true
                  />
                  <Stat
                      label="Offline runners"
                      value=Signal::derive(move || stats.get().offline)
                      description=Signal::stored(t("Needs runner attention"))
                  />
                  <Stat
                      label="Active runners"
                      value=Signal::derive(move || stats.get().active)
                      description=Signal::stored(t("Eligible to claim jobs"))
                      active=true
                  />
              </div>
              <guide::Trust />
              <Show when=move || !context.token.get().is_empty()>
                  <aside class="runner-notice runner-token" role="status">
                      <strong>{t("Runner token generated")}</strong>
                      <p>{t("Copy this token now. It is only shown after registration.")}</p>
                      <code tabindex="0">{move || context.token.get()}</code>
                  </aside>
              </Show>
              <div class="runner-between">
                  <button
                      type="button"
                      disabled=move || !context.can_admin()
                      on:click=move |_| context.composer.update(|open| *open = !*open)
                  >
                      <thaw::Icon icon=icondata_lu::LuPlus width="16px" height="16px" />
                      {move || t(
                          if context.composer.get() {
                              "Hide runner form"
                          } else {
                              "Register runner"
                          },
                      )}
                  </button>
                  <div class="runner-actions">
                      <span class="runner-badge" class:secondary=move || context.can_admin()>
                          {move || {
                              format!(
                                  "{}: {}",
                                  t("Role"),
                                  t(&model::role_label(&context.permissions.data.get())),
                              )
                          }}
                      </span>
                      <Reload query=context.rows />
                  </div>
              </div>
              <Show when=move || !context.permissions.loading.get() && !context.can_admin()>
                  <aside class="runner-notice">
                      {t(
                          "Your current project role can inspect runners, but cannot register or delete them.",
                      )}
                  </aside>
              </Show>
              <Show when=move || context.composer.get()>
                  <register::Register context=context />
              </Show>
              <list::Listing context=context runners=runners />
              <guide::Protocol />
              <variables::Variables context=context />
          </div>
      </section>
  }
}
#[component]
fn Stat(
  label: &'static str,
  value: Signal<usize>,
  description: Signal<String>,
  #[prop(default = false)] active: bool,
) -> impl IntoView {
  view! {
      <div class="runner-stat" class:active=active>
          <small>{t(label)}</small>
          <strong>{move || value.get()}</strong>
          <small>{move || description.get()}</small>
      </div>
  }
}
#[component]
fn Reload(query: Query) -> impl IntoView {
  view! {
      <button
          type="button"
          class="runner-ghost"
          on:click=move |_| query.refresh.update(|n| *n += 1)
      >
          <thaw::Icon icon=icondata_lu::LuRefreshCw width="14px" height="14px" />
          {t("Reload")}
      </button>
  }
}
#[component]
fn MutationError(mutation: Mutation) -> impl IntoView {
  view! {
      <Show when=move || !mutation.error.get().is_empty()>
          <p class="error" role="alert">
              {move || mutation.error.get()}
          </p>
      </Show>
  }
}
