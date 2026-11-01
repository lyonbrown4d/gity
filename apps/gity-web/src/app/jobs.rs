mod cards;
mod composer;
mod model;
mod output;
use crate::{api, i18n::t, ui::*};
use leptos::prelude::*;
use model::{Stats, Status};
#[derive(Clone, Copy)]
struct Context {
  path: StoredValue<String>,
  rows: Query,
  permissions: Query,
  can_write: Signal<bool>,
  composer: RwSignal<bool>,
  creation: Mutation,
  cancellation: Mutation,
}
#[component]
pub(super) fn Jobs(project_path: String, permissions: Query) -> impl IntoView {
  let rows = query(format!("{project_path}/jobs"));
  let context = Context {
    path: StoredValue::new(project_path),
    rows,
    permissions,
    can_write: Signal::derive(move || {
      allowed(
        &permissions.data.get(),
        "project.jobs.write",
        "job_write",
        30,
      )
    }),
    composer: RwSignal::new(false),
    creation: Mutation::new(rows.refresh),
    cancellation: Mutation::new(rows.refresh),
  };
  let stats = Memo::new(move |_| Stats::from_rows(&rows.data.get()));
  view! {
      <section class="jobs-panel account-card card-enter">
          <header class="jobs-header">
              <h2>{t("Jobs")}</h2>
              <p>{t("Inspect and enqueue project background jobs.")}</p>
          </header>
          <div class="jobs-body">
              <div class="jobs-stats">
                  {[Status::Pending, Status::Running, Status::Succeeded, Status::Failed]
                      .into_iter()
                      .map(|status| view! { <cards::Stat status=status stats=stats /> })
                      .collect_view()}
              </div>
              <aside class="jobs-notice jobs-queue-guide">
                  <strong>{t("Job diagnostics guide")}</strong>
                  <p>{move || t(stats.get().diagnostic())}</p>
                  <div class="jobs-badges">
                      <span class="badge" class:outline=move || stats.get().failed == 0>
                          {t("Trace")}
                          ": "
                          {t("pipeline job detail")}
                      </span>
                      <span class="badge outline">
                          {t("Artifacts")}": "{t("download after upload")}
                      </span>
                      <span class="badge outline">
                          {t("Empty logs")}": "
                          {t("not claimed, not flushed, or no stored output")}
                      </span>
                  </div>
              </aside>
              <div class="jobs-toolbar">
                  <button
                      type="button"
                      disabled=move || !context.can_write.get()
                      class:secondary=move || context.composer.get()
                      on:click=move |_| context.composer.update(|value| *value = !*value)
                  >
                      <thaw::Icon icon=icondata_lu::LuPlay width="16px" height="16px" />
                      {move || t(
                          if context.composer.get() { "Hide new job form" } else { "New job" },
                      )}
                  </button>
                  <div>
                      <span class="badge" class:outline=move || !context.can_write.get()>
                          {t("Role")}
                          ": "
                          {move || t(&role_label(&context.permissions.data.get()))}
                      </span>
                      <button
                          type="button"
                          class="jobs-reload"
                          on:click=move |_| context.rows.refresh.update(|n| *n += 1)
                      >
                          <thaw::Icon icon=icondata_lu::LuRefreshCw width="16px" height="16px" />
                          {t("Reload")}
                      </button>
                  </div>
              </div>
              <Show when=move || !context.permissions.loading.get() && !context.can_write.get()>
                  <aside class="jobs-notice">
                      {t(
                          "Your current project role can inspect jobs, but cannot enqueue or cancel them.",
                      )}
                  </aside>
              </Show>
              <Show when=move || context.composer.get()>
                  <composer::Composer context=context />
              </Show>
              <QueryError q=rows />
              <Show when=move || !context.cancellation.error.get().is_empty()>
                  <p class="error" role="alert">
                      {move || context.cancellation.error.get()}
                  </p>
              </Show>
              <div class="jobs-list">
                  <Show when=move || rows.loading.get() && rows.data.get().is_null()>
                      <div class="jobs-skeleton">
                          {(0..3)
                              .map(|_| view! { <div class="loading-placeholder"></div> })
                              .collect_view()}
                      </div>
                  </Show>
                  <Show when=move || {
                      !rows.loading.get() && rows.error.get().is_empty()
                          && api::list(&rows.data.get()).is_empty()
                  }>
                      <aside class="jobs-notice jobs-empty">
                          <strong>{t("No jobs in the queue")}</strong>
                          <p>
                              {t(
                                  "Enqueue a manual job to validate runner connectivity, or wait for a pipeline to create jobs automatically.",
                              )}
                          </p>
                          <div class="jobs-badges">
                              <button
                                  type="button"
                                  class="primary"
                                  disabled=move || !context.can_write.get()
                                  on:click=move |_| context.composer.set(true)
                              >
                                  <thaw::Icon
                                      icon=icondata_lu::LuPlay
                                      width="16px"
                                      height="16px"
                                  />
                                  {t("Enqueue first job")}
                              </button>
                              <Show when=move || !context.can_write.get()>
                                  <span class="badge outline">{t("Read-only role")}</span>
                              </Show>
                          </div>
                      </aside>
                  </Show>
                  {move || {
                      api::list(&rows.data.get())
                          .into_iter()
                          .map(|job| view! { <cards::JobCard context=context job=job /> })
                          .collect_view()
                  }}
              </div>
          </div>
      </section>
  }
}
