use super::*;
use serde_json::{Value, json};
#[component]
pub(super) fn Badge(status: Status) -> impl IntoView {
  let icon = match status {
    Status::Pending => icondata_lu::LuClock3,
    Status::Running => icondata_lu::LuLoaderCircle,
    Status::Succeeded => icondata_lu::LuCircleCheck,
    Status::Failed => icondata_lu::LuCircleX,
    Status::Cancelled => icondata_lu::LuBan,
  };
  view! {
      <span class=format!("badge job-status {}", status.key())>
          <span class:job-spinning=status == Status::Running>
              <thaw::Icon icon=icon width="12px" height="12px" />
          </span>
          {t(status.key())}
      </span>
  }
}
#[component]
pub(super) fn Stat(status: Status, stats: Memo<Stats>) -> impl IntoView {
  let label = match status {
    Status::Pending => "Pending",
    Status::Running => "Running",
    Status::Succeeded => "Succeeded",
    _ => "Failed",
  };
  let description = move || {
    let active = stats.get().count(status) > 0;
    t(match (status, active) {
      (Status::Pending, true) => "Waiting for a runner",
      (Status::Pending, false) => "Queue is clear",
      (Status::Running, true) => "Currently executing",
      (Status::Running, false) => "No active jobs",
      (Status::Succeeded, true) => "Completed successfully",
      (Status::Succeeded, false) => "No successful jobs yet",
      (_, true) => "Needs attention",
      (_, false) => "No failures",
    })
  };
  view! {
      <div class=format!("job-stat {}", status.key())>
          <div>
              <small>{t(label)}</small>
              <Badge status=status />
          </div>
          <strong>{move || stats.get().count(status)}</strong>
          <small>{description}</small>
      </div>
  }
}
#[component]
pub(super) fn JobCard(context: Context, job: Value) -> impl IntoView {
  let status = Status::parse(&api::text(&job, "status"));
  let id = api::text(&job, "id");
  let cancel_id = id.clone();
  let worker = api::text(&job, "locked_by");
  let date = api::text(&job, "updated_at");
  let error = api::text(&job, "last_error");
  let kind = api::text(&job, "kind");
  view! {
      <article class=format!("job-card {}", status.key())>
          <header>
              <div>
                  <div>
                      <strong>{format!("#{id}")}</strong>
                      <span class="badge outline">
                          {if kind.is_empty() { "noop".into() } else { kind }}
                      </span>
                      <Badge status=status />
                  </div>
                  <small>{t(status.description())}</small>
              </div>
              {(!status.terminal())
                  .then(|| {
                      view! {
                          <button
                              type="button"
                              disabled=move || {
                                  !context.can_write.get() || context.cancellation.busy.get()
                              }
                              on:click=move |_| {
                                  if context.can_write.get_untracked() {
                                      context
                                          .cancellation
                                          .run(
                                              "POST",
                                              format!(
                                                  "{}/jobs/{}/cancel",
                                                  context.path.get_value(),
                                                  api::enc(&cancel_id),
                                              ),
                                              json!({}),
                                              None,
                                          );
                                  }
                              }
                          >
                              <thaw::Icon icon=icondata_lu::LuBan width="16px" height="16px" />
                              {t("Cancel job")}
                          </button>
                      }
                  })}
          </header>
          <dl class="job-metadata">
              <Meta
                  label="Attempts"
                  value=format!(
                      "{}/{}",
                      job["attempts"].as_u64().unwrap_or_default(),
                      job["max_attempts"].as_u64().filter(|value| *value > 0).unwrap_or(1),
                  )
              />
              <Meta
                  label="Worker"
                  value=if worker.is_empty() { t("Unassigned") } else { worker }
              />
              <Meta
                  label="Updated"
                  value=if date.is_empty() { t("Not updated") } else { relative_time(&date) }
              />
          </dl>
          {(!error.is_empty())
              .then(|| {
                  view! {
                      <div class="job-last-error" role="alert">
                          {error}
                      </div>
                  }
              })}
          <output::Output job=job status=status />
      </article>
  }
}
#[component]
fn Meta(label: &'static str, value: String) -> impl IntoView {
  view! {
      <div>
          <dt>{t(label)}</dt>
          <dd title=value.clone()>{value.clone()}</dd>
      </div>
  }
}
