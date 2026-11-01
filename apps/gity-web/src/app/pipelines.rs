mod composer;
mod detail;
mod job_detail;
mod jobs;
mod listing;
mod messages;
mod model;
mod overview;
use crate::{api, i18n::t, ui::*};
use leptos::prelude::*;
use model::{Job, Pipeline, State};
use serde_json::Value;

#[derive(Clone, Copy)]
struct Context {
  path: StoredValue<String>,
  permissions: Query,
  rows: Query,
  detail: Query,
  trace: Query,
  artifacts: Query,
  selected: RwSignal<String>,
  selected_job: RwSignal<String>,
  updates: RwSignal<u64>,
  pipelines: Memo<Vec<Pipeline>>,
  jobs: Memo<Vec<Job>>,
  composer: RwSignal<bool>,
  source: RwSignal<String>,
  reference: RwSignal<String>,
  commit: RwSignal<String>,
  config: RwSignal<String>,
  content: RwSignal<String>,
  lint_result: RwSignal<String>,
  cancelling: RwSignal<bool>,
  retrying: RwSignal<bool>,
  refreshing: RwSignal<bool>,
  cancelling_job: RwSignal<bool>,
  retrying_job: RwSignal<bool>,
}
impl Context {
  fn can_write(self) -> bool {
    self
      .permissions
      .data
      .with(|value| allowed(value, "project.jobs.write", "job_write", 30))
  }
  fn select(self, id: String) {
    if self.selected.get_untracked() != id {
      self.selected_job.set(String::new());
      self.selected.set(id);
    }
  }
  fn pipeline(self) -> Option<Pipeline> {
    let selected = self.selected.get();
    self
      .detail
      .data
      .with(|value| {
        (api::text(&value["pipeline"], "id") == selected && !selected.is_empty())
          .then(|| Pipeline::from(&value["pipeline"]))
      })
      .or_else(|| {
        self
          .pipelines
          .get()
          .into_iter()
          .find(|row| row.id == selected)
      })
  }
  fn job(self) -> Option<Job> {
    let selected = self.selected_job.get();
    self.jobs.get().into_iter().find(|job| job.id == selected)
  }
}

#[component]
pub(super) fn Pipelines(
  project_path: String,
  permissions: Query,
  default_branch: Signal<String>,
) -> impl IntoView {
  let path = StoredValue::new(project_path.clone());
  let selected = RwSignal::new(String::new());
  let selected_job = RwSignal::new(String::new());
  let rows = query(format!("{project_path}/pipelines"));
  let detail = query_fn(
    move || {
      let id = selected.get();
      if id.is_empty() {
        String::new()
      } else {
        format!("{}/pipelines/{id}", path.get_value())
      }
    },
    true,
  );
  let trace = query_fn(move || job_path(path, selected_job, "trace"), true);
  let artifacts = query_fn(move || job_path(path, selected_job, "artifacts"), true);
  let pipelines = Memo::new(move |_| rows.data.with(model::pipelines));
  let jobs = Memo::new(move |_| {
    let id = selected.get();
    detail.data.with(|value| {
      if api::text(&value["pipeline"], "id") != id || id.is_empty() {
        return Vec::new();
      }
      value["jobs"]
        .as_array()
        .into_iter()
        .flatten()
        .map(Job::from)
        .collect::<Vec<_>>()
    })
  });
  let branch = default_branch.get_untracked();
  let context = Context {
    path,
    permissions,
    rows,
    detail,
    trace,
    artifacts,
    selected,
    selected_job,
    pipelines,
    jobs,
    updates: RwSignal::new(0),
    composer: RwSignal::new(false),
    source: RwSignal::new("web".into()),
    reference: RwSignal::new(if branch.is_empty() {
      "main".into()
    } else {
      branch
    }),
    commit: RwSignal::new(String::new()),
    config: RwSignal::new(".gity-ci.yml".into()),
    content: RwSignal::new(composer::DEFAULT_CONFIG.into()),
    lint_result: RwSignal::new(String::new()),
    cancelling: RwSignal::new(false),
    retrying: RwSignal::new(false),
    refreshing: RwSignal::new(false),
    cancelling_job: RwSignal::new(false),
    retrying_job: RwSignal::new(false),
  };
  Effect::new(move |_| {
    if rows.loading.get() {
      return;
    }
    let current = selected.get_untracked();
    let list = pipelines.get();
    if !list.iter().any(|row| row.id == current) {
      context.select(list.first().map(|row| row.id.clone()).unwrap_or_default());
    }
  });
  Effect::new(move |_| {
    let list = jobs.get();
    let current = selected_job.get_untracked();
    if !list.iter().any(|row| row.id == current) {
      selected_job.set(list.first().map(|row| row.id.clone()).unwrap_or_default());
    }
  });
  Effect::new(move |previous: Option<u64>| {
    let update = context.updates.get();
    if previous.is_some() {
      for query in [rows, detail, trace, artifacts] {
        query.refresh.update(|n| *n += 1);
      }
    }
    update
  });
  view! {
      <section class="pipelines-panel account-card card-enter">
          <header class="pipeline-header">
              <h2>{t("Pipelines")}</h2>
              <p>{t("Track CI pipelines triggered by pushes and merge requests.")}</p>
          </header>
          <div class="pipeline-body">
              <overview::Overview context=context />
              <Show when=move || !context.permissions.loading.get() && !context.can_write()>
                  <Notice
                      title="Read-only CI access"
                      message="Your current project role can inspect pipelines, but cannot create, cancel, retry, or refresh pipeline state."
                  />
              </Show>
              <div class="pipeline-between">
                  <button
                      type="button"
                      disabled=move || !context.can_write()
                      on:click=move |_| context.composer.update(|open| *open = !*open)
                  >
                      <thaw::Icon icon=icondata_lu::LuPlay width="16px" height="16px" />
                      {move || t(
                          if context.composer.get() {
                              "Hide pipeline form"
                          } else {
                              "New pipeline"
                          },
                      )}
                  </button>
                  <div class="pipeline-actions">
                      <span class="pipeline-badge">
                          {move || {
                              format!(
                                  "{}: {}",
                                  t("Role"),
                                  t(&role_label(&permissions.data.get())),
                              )
                          }}
                      </span>
                      <Reload query=rows label="Reload" />
                  </div>
              </div>
              <Show when=move || context.composer.get()>
                  <composer::Composer context=context />
              </Show>
              <div class="pipeline-workspace">
                  <listing::Listing context=context />
                  <detail::Detail context=context />
              </div>
          </div>
      </section>
  }
}
fn job_path(path: StoredValue<String>, selected: RwSignal<String>, suffix: &str) -> String {
  let id = selected.get();
  if id.is_empty() {
    String::new()
  } else {
    format!("{}/jobs/{id}/{suffix}", path.get_value())
  }
}
#[component]
fn Badge(state: State) -> impl IntoView {
  let icon = match state {
    State::Running => icondata_lu::LuLoaderCircle,
    State::Succeeded => icondata_lu::LuCircleCheck,
    State::Failed => icondata_lu::LuCircleX,
    State::Cancelled => icondata_lu::LuBan,
    _ => icondata_lu::LuClock3,
  };
  view! {
      <span class=format!("pipeline-badge {}", state.key())>
          <thaw::Icon icon=icon width="12px" height="12px" />
          {t(state.key())}
      </span>
  }
}
#[component]
fn Notice(title: &'static str, message: &'static str) -> impl IntoView {
  view! {
      <aside class="pipeline-notice">
          <strong>{t(title)}</strong>
          <p>{t(message)}</p>
      </aside>
  }
}
#[component]
fn Reload(query: Query, label: &'static str) -> impl IntoView {
  view! {
      <button
          type="button"
          class="pipeline-ghost"
          on:click=move |_| query.refresh.update(|n| *n += 1)
      >
          <thaw::Icon icon=icondata_lu::LuRefreshCw width="14px" height="14px" />
          {t(label)}
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
