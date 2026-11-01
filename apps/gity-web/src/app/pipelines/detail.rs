use super::*;

#[component]
pub(super) fn Detail(context: Context) -> impl IntoView {
  view! {
      <div class="pipeline-box pipeline-detail pipeline-stack">
          <Show when=move || context.pipeline().is_none()>
              <Notice
                  title="Select a pipeline"
                  message="Choose a pipeline from the list to inspect jobs, logs, artifacts, and retry controls."
              />
          </Show>
          {move || {
              context
                  .pipeline()
                  .map(|pipeline| view! { <Header context=context pipeline=pipeline /> })
          }}
          <QueryError q=context.detail />
          <Show when=move || context.pipeline().is_some()>
              <Diagnostics context=context />
              <jobs::Jobs context=context />
              {move || {
                  context
                      .job()
                      .map(|job| view! { <job_detail::JobDetail context=context job=job /> })
              }}
          </Show>
      </div>
  }
}
#[component]
fn Header(context: Context, pipeline: Pipeline) -> impl IntoView {
  let state = pipeline.state;
  let path = format!("{}/pipelines/{}", context.path.get_value(), pipeline.id);
  view! {
      <div class="pipeline-between pipeline-detail-heading">
          <div class="pipeline-stack">
              <div class="pipeline-actions">
                  <h3>{pipeline.title()}</h3>
                  <Badge state=state />
              </div>
              <p class="pipeline-muted">
                  {format!(
                      "{}: {} · {}: {} · {}: {}",
                      t("Source"),
                      pipeline.source,
                      t("Ref"),
                      pipeline.reference,
                      t("Commit"),
                      pipeline.short_commit(),
                  )}
              </p>
          </div>
          <div class="pipeline-actions">
              <Reload query=context.detail label="Reload pipeline" />
              <Action
                  label=t("Refresh state")
                  path=format!("{path}/refresh")
                  refresh=context.updates
                  busy_state=context.refreshing
                  disabled=Signal::derive(move || !context.can_write())
              >
                  <thaw::Icon icon=icondata_lu::LuRefreshCw width="14px" height="14px" />
                  {move || t(
                      if context.refreshing.get() { "Refreshing..." } else { "Refresh state" },
                  )}
              </Action>
              <Show when=move || !state.terminal()>
                  <Action
                      label=t("Cancel pipeline")
                      path=format!(
                          "{}/pipelines/{}/cancel",
                          context.path.get_value(),
                          context.selected.get_untracked(),
                      )
                      refresh=context.updates
                      busy_state=context.cancelling
                      disabled=Signal::derive(move || !context.can_write())
                  >
                      <thaw::Icon icon=icondata_lu::LuBan width="14px" height="14px" />
                      {move || t(
                          if context.cancelling.get() {
                              "Cancelling..."
                          } else {
                              "Cancel pipeline"
                          },
                      )}
                  </Action>
              </Show>
              <Show when=move || state != State::Running>
                  <Action
                      label=t("Retry pipeline")
                      path=format!(
                          "{}/pipelines/{}/retry",
                          context.path.get_value(),
                          context.selected.get_untracked(),
                      )
                      refresh=context.updates
                      busy_state=context.retrying
                      disabled=Signal::derive(move || !context.can_write() || !state.terminal())
                  >
                      <thaw::Icon icon=icondata_lu::LuRepeat2 width="14px" height="14px" />
                      {move || t(
                          if context.retrying.get() { "Retrying..." } else { "Retry pipeline" },
                      )}
                  </Action>
              </Show>
          </div>
      </div>
      <div class="pipeline-metadata">
          <Meta label="Config" value=pipeline.config />
          <Meta label="Started" value=time(&pipeline.started) />
          <Meta label="Finished" value=time(&pipeline.finished) />
      </div>
  }
}
#[component]
fn Meta(label: &'static str, value: String) -> impl IntoView {
  view! {
      <div class="pipeline-meta">
          <small>{t(label)}</small>
          <p>{if value.is_empty() { t("N/A") } else { value }}</p>
      </div>
  }
}
fn time(value: &str) -> String {
  if value.is_empty() {
    t("N/A")
  } else {
    relative_time(value)
  }
}
#[component]
fn Diagnostics(context: Context) -> impl IntoView {
  let summary = Memo::new(move |_| context.jobs.with(|jobs| messages::Summary::new(jobs)));
  view! {
      <aside
          class="pipeline-notice"
          class:destructive=move || { summary.get().failed > 0 }
          class:pipeline-empty=move || summary.get().failed == 0 && summary.get().blocked == 0
      >
          <strong>{t("Pipeline diagnostics")}</strong>
          <div class="pipeline-stack">
              <div class="pipeline-summary-pills">
                  <Pill label="Jobs" count=Signal::derive(move || summary.get().total) />
                  <Pill
                      label="Failed"
                      count=Signal::derive(move || summary.get().failed)
                      urgent=true
                  />
                  <Pill
                      label="Blocked"
                      count=Signal::derive(move || summary.get().blocked)
                      urgent=true
                  />
                  <Pill
                      label="Artifacts expected"
                      count=Signal::derive(move || summary.get().artifacts)
                  />
              </div>
              <p>
                  {move || {
                      context
                          .pipeline()
                          .map(|pipeline| t(
                              summary.get().guidance(&pipeline, context.can_write()),
                          ))
                          .unwrap_or_default()
                  }}
              </p>
              <Show when=move || summary.get().failed + summary.get().blocked != 0>
                  <div class="pipeline-actions">
                      {move || {
                          context
                              .jobs
                              .get()
                              .into_iter()
                              .filter(|job| matches!(job.state, State::Failed | State::Blocked))
                              .take(3)
                              .map(|job| {
                                  view! {
                                      <span
                                          class="pipeline-badge"
                                          class:failed=job.state == State::Failed
                                      >
                                          {format!("{}: {}", job.title(), t(job.state.key()))}
                                      </span>
                                  }
                              })
                              .collect_view()
                      }}
                  </div>
              </Show>
          </div>
      </aside>
  }
}
#[component]
fn Pill(
  label: &'static str,
  count: Signal<usize>,
  #[prop(default = false)] urgent: bool,
) -> impl IntoView {
  view! {
      <div class="pipeline-pill" class:urgent=move || { urgent && count.get() > 0 }>
          <small>{t(label)}</small>
          <strong>{move || count.get()}</strong>
      </div>
  }
}
