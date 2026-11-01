use super::*;

#[component]
pub(super) fn Overview(context: Context) -> impl IntoView {
  view! {
      <div class="pipeline-stats">
          <Stat
              context=context
              state=State::Pending
              label="Pending"
              present="Waiting for execution"
              empty="No queued pipelines"
          />
          <Stat
              context=context
              state=State::Running
              label="Running"
              present="Jobs are executing"
              empty="No active pipelines"
          />
          <Stat
              context=context
              state=State::Succeeded
              label="Succeeded"
              present="Healthy completions"
              empty="No successes yet"
          />
          <Stat
              context=context
              state=State::Failed
              label="Failed"
              present="Needs investigation"
              empty="No failures"
          />
      </div>
      <aside class="pipeline-notice">
          <strong>{t("Pipeline status summary")}</strong>
          <div class="pipeline-stack" style="gap:8px">
              <p>{move || t(messages::activity(&context.pipelines.get()))}</p>
              <div class="pipeline-actions">
                  <Count context=context state=State::Failed label="Failures" />
                  <Count context=context state=State::Running label="Running" />
                  <Count context=context state=State::Pending label="Queued" />
              </div>
          </div>
      </aside>
  }
}
#[component]
fn Stat(
  context: Context,
  state: State,
  label: &'static str,
  present: &'static str,
  empty: &'static str,
) -> impl IntoView {
  let count = move || {
    context
      .pipelines
      .with(|rows| rows.iter().filter(|row| row.state == state).count())
  };
  view! {
      <div
          class="pipeline-stat"
          class:failed=state == State::Failed
          class:running=state == State::Running
      >
          <div class="pipeline-between">
              <small>{t(label)}</small>
              <Badge state=state />
          </div>
          <strong>{count}</strong>
          <small>{move || t(if count() > 0 { present } else { empty })}</small>
      </div>
  }
}
#[component]
fn Count(context: Context, state: State, label: &'static str) -> impl IntoView {
  view! {
      <span class="pipeline-badge" class:failed=state == State::Failed>
          {move || {
              format!(
                  "{}: {}",
                  t(label),
                  context
                      .pipelines
                      .with(|rows| rows.iter().filter(|row| row.state == state).count()),
              )
          }}
      </span>
  }
}
