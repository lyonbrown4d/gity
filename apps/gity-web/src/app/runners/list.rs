use super::*;
use model::Runner;

#[component]
pub(super) fn Listing(context: Context, runners: Memo<Vec<Runner>>) -> impl IntoView {
  let initial_loading =
    Signal::derive(move || context.rows.loading.get() && context.rows.data.get().is_null());
  view! {
      <div class="runner-list">
          <Show when=move || initial_loading.get()>
              <div class="runner-skeleton" role="status" aria-label=t("Loading runners...")></div>
              <div class="runner-skeleton"></div>
          </Show>
          <Show when=move || !context.rows.error.get().is_empty()>
              <Status q=context.rows />
          </Show>
          <Show when=move || {
              !initial_loading.get() && context.rows.error.get().is_empty()
                  && runners.get().is_empty()
          }>
              <Empty context=context />
          </Show>
          {move || {
              runners
                  .get()
                  .into_iter()
                  .map(|runner| view! { <Entry context=context runner=runner /> })
                  .collect_view()
          }}
      </div>
  }
}
#[component]
fn Empty(context: Context) -> impl IntoView {
  view! {
      <aside class="runner-notice runner-empty">
          <strong>{t("No runners registered")}</strong>
          <p>
              {t(
                  "Register a project runner on a trusted host before pipelines can claim CI jobs on this repository. Add tags that match your CI config, then verify the runner sends heartbeats.",
              )}
          </p>
          <div class="runner-actions">
              <button
                  type="button"
                  class="primary"
                  disabled=move || !context.can_admin()
                  on:click=move |_| context.composer.set(true)
              >
                  <thaw::Icon icon=icondata_lu::LuPlus width="14px" height="14px" />
                  {t("Register first runner")}
              </button>
              <Show when=move || !context.can_admin()>
                  <span class="runner-badge">{t("Runner admin required")}</span>
              </Show>
          </div>
      </aside>
  }
}
#[component]
fn Entry(context: Context, runner: Runner) -> impl IntoView {
  let name = runner.name.clone();
  let health = if runner.online && !runner.last_contact.is_empty() {
    format!(
      "{}: {}",
      t("Last heartbeat"),
      relative_time(&runner.last_contact)
    )
  } else if runner.online {
    t(
      "Online, but no heartbeat timestamp is available. Verify runtime toolchains if jobs are still failing.",
    )
  } else {
    t("Offline runner. Check the runner process, trusted host network, and token configuration.")
  };
  let trusted = if runner.active {
    "Can claim jobs; verify this host is isolated for CI workloads."
  } else {
    "Paused runners should not claim jobs until reactivated."
  };
  let runtime = if runner.online {
    "Heartbeat is present; verify toolchains and caches if jobs still fail."
  } else {
    "No recent heartbeat; restart the runner process or rotate the token if needed."
  };
  let tags = if runner.tags.is_empty() {
    t("No tags configured; tagged jobs will not match this runner.")
  } else {
    runner.tags.join(", ")
  };
  view! {
      <article
          class="runner-entry runner-box runner-stack"
          class:online=runner.online
          class:paused=!runner.active
      >
          <div class="runner-between">
              <div class="runner-entry-body">
                  <div class="runner-actions">
                      <strong>{runner.name}</strong>
                      <span class="runner-badge" class:online=runner.online>
                          <thaw::Icon icon=icondata_lu::LuActivity width="12px" height="12px" />
                          {t(if runner.online { "online" } else { "offline" })}
                      </span>
                      <span class="runner-badge" class:secondary=runner.active>
                          {t(if runner.active { "active" } else { "paused" })}
                      </span>
                  </div>
                  {if runner.description.is_empty() {
                      None
                  } else {
                      Some(view! { <p>{runner.description}</p> })
                  }}
                  <small>{health}</small>
                  <div class="runner-actions">
                      <span class="runner-badge">{format!("#{}", runner.id)}</span>
                      {runner
                          .tags
                          .into_iter()
                          .map(|tag| view! { <span class="runner-badge">{tag}</span> })
                          .collect_view()}
                  </div>
                  <div class="runner-guide-grid">
                      <guide::Item label=t("Trusted host") description=t(trusted) />
                      <guide::Item label=t("Runtime health") description=t(runtime) />
                      <guide::Item label=t("Tags") description=tags />
                  </div>
              </div>
              <Action
                  label="Delete".into()
                  path=format!("{}/runners/{}", context.path.get_value(), runner.id)
                  method="DELETE".into()
                  refresh=context.rows.refresh
                  confirm=true
                  confirmation_title=t("Delete runner \"{name}\"?").replace("{name}", &name)
                  confirmation_description="This action cannot be undone.".into()
                  confirmation_label="Delete".into()
                  busy_state=context.deleting
                  disabled=Signal::derive(move || !context.can_admin() || context.deleting.get())
              >
                  <thaw::Icon icon=icondata_lu::LuTrash2 width="14px" height="14px" />
                  {t("Delete")}
              </Action>
          </div>
      </article>
  }
}
