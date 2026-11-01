use crate::{api, i18n::t, ui::*};
use leptos::prelude::*;

#[component]
pub(super) fn Commits(project_path: String) -> impl IntoView {
  let branches = query(format!("{project_path}/repository/branches"));
  let selected = RwSignal::new("all".to_string());
  let commits = query_fn(
    move || {
      let filter = selected.get();
      format!(
        "{project_path}/repository/commits?limit=50{}",
        if filter == "all" {
          String::new()
        } else {
          format!("&branch_name={}", api::enc(&filter))
        }
      )
    },
    true,
  );
  view! {
      <section class="repository-activity account-card card-enter">
          <header class="commit-header">
              <div>
                  <h2>{t("Commits")}</h2>
                  <p>{t("Recent commit activity in this project.")}</p>
              </div>
              <label for="commit-branch-filter">
                  {t("Branch")}
                  <select
                      id="commit-branch-filter"
                      aria-label=t("Branch")
                      prop:value=move || selected.get()
                      on:change=move |event| selected.set(event_target_value(&event))
                  >
                      <option value="all">{t("All branches")}</option>
                      {move || {
                          api::list(&branches.data.get())
                              .into_iter()
                              .map(|branch| {
                                  let name = api::text(&branch, "name");
                                  view! { <option value=name.clone()>{name.clone()}</option> }
                              })
                              .collect_view()
                      }}
                  </select>
              </label>
          </header>
          <div class="repository-activity-body commit-list">
              <Status q=branches />
              <Show when=move || commits.loading.get()>
                  <p class="muted" role="status">
                      {t("Loading commits...")}
                  </p>
              </Show>
              <Show when=move || !commits.error.get().is_empty()>
                  <p class="error" role="alert">
                      {move || commits.error.get()}
                  </p>
              </Show>
              {move || {
                  api::list(&commits.data.get())
                      .into_iter()
                      .map(|commit| {
                          view! {
                              <article class="repository-commit">
                                  <div>
                                      <span class="badge outline">
                                          {api::text(&commit, "commit_sha")
                                              .chars()
                                              .take(8)
                                              .collect::<String>()}
                                      </span>
                                      <span class="badge">
                                          {api::text(&commit, "branch_name")}
                                      </span>
                                  </div>
                                  <p>{api::text(&commit, "message")}</p>
                                  <small>
                                      {api::text(&commit, "author_user_id")}" · "
                                      {format_timestamp(&api::text(&commit, "created_at"))}
                                  </small>
                              </article>
                          }
                      })
                      .collect_view()
              }}
              <Show when=move || {
                  !commits.loading.get() && commits.error.get().is_empty()
                      && api::list(&commits.data.get()).is_empty()
              }>
                  <p class="muted">{t("No commits found.")}</p>
              </Show>
          </div>
      </section>
  }
}
