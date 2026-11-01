use super::project::CloneUrl;
use crate::{api, markdown::Markdown, ui::*};
use leptos::prelude::*;
use leptos_router::components::A;
use serde_json::Value;
#[component]
pub(super) fn Overview(
  project_path: String,
  project: Value,
  base: String,
  permissions: Value,
) -> impl IntoView {
  let capabilities = expect_context::<Query>();
  let readme = query(format!(
    "{project_path}/repository/readme?branch_name={}",
    api::enc(&api::text(&project, "default_branch"))
  ));
  let commits = query(format!("{project_path}/repository/commits"));
  let branches = query(format!("{project_path}/repository/branches"));
  let activities = [
    ("issues", "Issues", "Open issues", "View issues"),
    (
      "merge-requests",
      "Merge requests",
      "Requests awaiting review",
      "Open merge requests",
    ),
    (
      "pipelines",
      "Pipelines",
      "Current delivery activity",
      "View pipelines",
    ),
    ("jobs", "Jobs", "Project task queue", "View jobs"),
    (
      "runners",
      "Runners",
      "Available execution agents",
      "View runners",
    ),
    ("wiki/pages", "Wiki", "Project documentation", "Open wiki"),
  ];
  let releases = query(format!("{project_path}/releases"));
  let pulse = activities
    .into_iter()
    .map(|(path, title, detail, action)| {
      (
        path,
        title,
        detail,
        action,
        query(format!("{project_path}/{path}")),
      )
    })
    .collect::<Vec<_>>();
  let package_path = project_path.clone();
  let packages = query_fn(
    move || {
      if capabilities.data.get()["package_registry"].as_bool() == Some(true) {
        format!("{package_path}/packages")
      } else {
        String::new()
      }
    },
    true,
  );
  let code = format!("{base}?tab=code");
  let all_commits = format!("{base}?tab=commits");
  let packages_link = format!("{base}?tab=packages");
  let releases_link = format!("{base}?tab=releases");
  view! {
      <section class="overview-workspace">
          <div class="workspace-panel-header">
              <div>
                  <h2>{crate::i18n::t("Repository workspace")}</h2>
                  <p class="mono muted">{api::text(&project, "clone_http_url")}</p>
              </div>
              <div class="toolbar">
                  <CloneUrl value=api::text(&project, "clone_http_url") />
                  <A href=code.clone() {..} class="button primary">
                      {crate::i18n::t("‹› Browse code")}
                  </A>
              </div>
          </div>
          <div class="repository-facts">
              <RepoFact
                  label="Default branch"
                  value=Signal::stored(api::text(&project, "default_branch"))
                  detail=Signal::derive(move || {
                      format!(
                          "{} protected",
                          api::list(&branches.data.get())
                              .iter()
                              .filter(|v| v["is_protected"] == true)
                              .count(),
                      )
                  })
              />
              <RepoFact
                  label="Access role"
                  value=Signal::stored(api::text(&permissions, "role"))
                  detail=Signal::stored("Project access".into())
              />
              <RepoFact
                  label="Latest commit"
                  value=Signal::derive(move || {
                      api::list(&commits.data.get())
                          .first()
                          .map(|v| api::text(v, "commit_sha").chars().take(8).collect::<String>())
                          .unwrap_or("None".into())
                  })
                  detail=Signal::derive(move || {
                      api::list(&commits.data.get())
                          .first()
                          .map(|v| api::text(v, "created_at"))
                          .unwrap_or("No commits yet".into())
                  })
              />
              <RepoFact
                  label="Visibility"
                  value=Signal::stored(api::text(&project, "visibility"))
                  detail=Signal::stored(api::text(&project, "status"))
              />
          </div>
          <div class="overview-columns">
              <main class="readme-canvas">
                  <div class="workspace-strip">
                      <div>
                          <strong>{crate::i18n::t("▤ README")}</strong>
                          <small>{move || api::text(&readme.data.get(), "path")}</small>
                      </div>
                      <A href=code.clone() {..} class="button ghost">
                          {crate::i18n::t("View source →")}
                      </A>
                  </div>
                  <div class="readme-content">
                      <Status q=readme />
                      {move || {
                          let data = readme.data.get();
                          let content = api::text(&data, "content");
                          if content.is_empty() {
                              view! {
                                  <div class="readme-empty">
                                      <strong>{crate::i18n::t("No README found")}</strong>
                                      <p>
                                          {crate::i18n::t(
                                              "Add a README to turn this project home into a useful landing page for contributors.",
                                          )}
                                      </p>
                                      <A href=code.clone() {..} class="button">
                                          {crate::i18n::t("Open repository files")}
                                      </A>
                                  </div>
                              }
                                  .into_any()
                          } else {
                              view! { <Markdown content=content /> }.into_any()
                          }
                      }}
                  </div>
              </main>
              <aside class="repository-pulse">
                  <div class="workspace-strip">
                      <div>
                          <strong>{crate::i18n::t("Repository pulse")}</strong>
                          <small>{crate::i18n::t("Current project activity")}</small>
                      </div>
                      <span class="badge">{move || {
                          if capabilities.data.get()["package_registry"].as_bool() == Some(true) {
                              "8"
                          } else {
                              "7"
                          }
                      }}</span>
                  </div>
                  {pulse
                      .into_iter()
                      .map(|(path, title, detail, action, q)| {
                          let tab = if path == "wiki/pages" { "wiki" } else { path };
                          view! {
                              <A
                                  href=format!("{base}?tab={tab}")
                                  {..}
                                  class="pulse-row"
                                  aria-label=action
                              >
                                  <span>
                                      <strong>{crate::i18n::t(title)}</strong>
                                      <small>{crate::i18n::t(detail)}</small>
                                  </span>
                                  <b>{move || api::list(&q.data.get()).len()}</b>
                              </A>
                          }
                      })
                      .collect_view()}
                  <Show when=move || {
                      capabilities.data.get()["package_registry"].as_bool() == Some(true)
                  }>
                      <A href=packages_link.clone() {..} class="pulse-row" aria-label="View packages">
                          <span>
                              <strong>{crate::i18n::t("Packages")}</strong>
                              <small>{crate::i18n::t("Published package registry")}</small>
                          </span>
                          <b>{move || api::list(&packages.data.get()).len()}</b>
                      </A>
                  </Show>
                  <A href=releases_link.clone() {..} class="pulse-row" aria-label="View releases">
                      <span>
                          <strong>{crate::i18n::t("Releases")}</strong>
                          <small>{crate::i18n::t("Published versions")}</small>
                      </span>
                      <b>{move || api::list(&releases.data.get()).len()}</b>
                  </A>
                  <div class="workspace-strip">
                      <strong>{crate::i18n::t("Recent commits")}</strong>
                      <A href=all_commits.clone() {..} class="button ghost">
                          {crate::i18n::t("View all")}
                      </A>
                  </div>
                  {move || {
                      api::list(&commits.data.get())
                          .into_iter()
                          .take(5)
                          .map(|commit| {
                              view! {
                                  <A href=all_commits.clone() {..} class="commit-summary">
                                      <span class="badge outline mono">
                                          {api::text(&commit, "commit_sha")
                                              .chars()
                                              .take(8)
                                              .collect::<String>()}
                                      </span>
                                      <small>{api::text(&commit, "branch_name")}</small>
                                      <strong>
                                          {api::text(&commit, "message")
                                              .lines()
                                              .next()
                                              .unwrap_or("")
                                              .to_string()}
                                      </strong>
                                      <p>
                                          {api::text(&commit, "author_user_id")}" · "
                                          {api::text(&commit, "created_at")}
                                      </p>
                                  </A>
                              }
                          })
                          .collect_view()
                  }}
                  <Show when=move || {
                      !commits.loading.get() && api::list(&commits.data.get()).is_empty()
                  }>
                      <p class="muted list-empty">{crate::i18n::t("No commits found.")}</p>
                  </Show>
              </aside>
          </div>
      </section>
  }
}
#[component]
fn RepoFact(label: &'static str, value: Signal<String>, detail: Signal<String>) -> impl IntoView {
  view! {
      <div>
          <span>{crate::i18n::t("◇")}</span>
          <dl>
              <dt>{crate::i18n::t(label)}</dt>
              <dd>
                  <strong>{move || value.get()}</strong>
                  <small>{move || detail.get()}</small>
              </dd>
          </dl>
      </div>
  }
}
