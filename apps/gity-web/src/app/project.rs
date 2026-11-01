use super::capabilities::PackageAvailabilityGate;
use super::overview::Overview;
use crate::{api, source::Source, ui::*};
use leptos::prelude::*;
use leptos_router::components::A;
#[component]
pub(super) fn Project(project_path: String, base: String, tab: String) -> impl IntoView {
  let q = query(project_path.clone());
  let orgs = query("/orgs".into());
  let permissions = query(format!("{project_path}/permissions"));
  let capabilities = expect_context::<Query>();
  let content_path = project_path.clone();
  view! {
      <div class="repository-breadcrumb">
          <A href="/app/projects">{crate::i18n::t("Projects")}</A>
          <span>{crate::i18n::t("›")}</span>
          <span>
              {move || {
                  let organization = api::text(&q.data.get(), "organization_id");
                  api::list(&orgs.data.get())
                      .into_iter()
                      .find(|org| api::text(org, "id") == organization)
                      .map(|org| api::text(&org, "name"))
                      .unwrap_or(organization)
              }}
          </span>
          <span>{crate::i18n::t("›")}</span>
          <strong>{move || api::text(&q.data.get(), "name")}</strong>
      </div>
      {move || {
          view! { <ProjectHeader project=q.data.get() permissions=permissions.data.get() /> }
      }}
      <Status q=q />
      {match tab.as_str() {
          "overview" => {
              view! {
                  {move || {
                      if q.loading.get() {
                          view! {
                              <SkeletonLines lines=2 />
                              <SkeletonCards cards=3 />
                          }
                              .into_any()
                      } else {
                          view! {
                              <Overview
                                  project_path=content_path.clone()
                                  project=q.data.get()
                                  base=base.clone()
                                  permissions=permissions.data.get()
                              />
                          }
                              .into_any()
                      }
                  }}
              }
                  .into_any()
          }
          "code" => {
              view! {
                  {move || {
                      if q.loading.get() {
                          return view! { <SkeletonRows rows=6 /> }.into_any();
                      }
                      let b = api::text(&q.data.get(), "default_branch");
                      view! {
                          <Source
                              project_path=content_path.clone()
                              project=q.data.get()
                              default_branch=if b.is_empty() { "main".into() } else { b }
                          />
                      }
                          .into_any()
                  }}
              }
                  .into_any()
          }
          "issues" => {
              view! {
                  <crate::issues::IssueBoard
                      path=format!("{content_path}/issues")
                      base=base.clone()
                  />
              }
                  .into_any()
          }
          "wiki" => {
              view! {
                  <super::wiki::Wiki project_path=content_path.clone() permissions=permissions />
              }
                  .into_any()
          }
          "merge-requests" => {
              view! {
                  <super::merge_requests::MergeRequests
                      project_path=content_path.clone()
                      permissions=permissions
                  />
              }
                  .into_any()
          }
          "branches" => {
              view! {
                  <super::branches::Branches
                      project_path=content_path.clone()
                      default_branch=Signal::derive(move || api::text(
                          &q.data.get(),
                          "default_branch",
                      ))
                      permissions=permissions
                  />
              }
                  .into_any()
          }
          "commits" => {
              view! { <super::commits::Commits project_path=content_path.clone() /> }.into_any()
          }
          "audit" => {
              view! { <super::audit::Audit project_path=content_path.clone() /> }.into_any()
          }
          "jobs" => {
              view! {
                  <super::jobs::Jobs project_path=content_path.clone() permissions=permissions />
              }
                  .into_any()
          }
          "runners" => {
              view! {
                  <super::runners::Runners
                      project_path=content_path.clone()
                      permissions=permissions
                  />
              }
                  .into_any()
          }
          "settings" => {
              view! {
                  <super::settings::Settings
                      project_path=content_path.clone()
                      project=q
                      permissions=permissions
                  />
              }
                  .into_any()
          }
          "packages" => {
              view! {
                  <PackageAvailabilityGate
                      capabilities=capabilities
                      project_path=content_path.clone()
                      permissions=permissions
                  />
              }
                  .into_any()
          }
          "releases" => {
              view! {
                  <super::releases::Releases
                      project_path=content_path.clone()
                      permissions=permissions
                  />
              }
                  .into_any()
          }
          "lfs" => {
              view! {
                  <super::lfs::Lfs
                      project_path=content_path.clone()
                      project=q.data
                      permissions=permissions
                  />
              }
                  .into_any()
          }
          "pipelines" => {
              view! {
                  <super::pipelines::Pipelines
                      project_path=content_path.clone()
                      permissions=permissions
                      default_branch=Signal::derive(move || api::text(
                          &q.data.get(),
                          "default_branch",
                      ))
                  />
              }
                  .into_any()
          }
          _ => view! { <p role="alert">{crate::i18n::t("Page not found")}</p> }.into_any(),
      }}
      <Show when=move || !permissions.error.get().is_empty()>
          <p class="error">{move || permissions.error.get()}</p>
      </Show>
  }
}

#[component]
pub(super) fn CloneUrl(
  value: String,
  #[prop(default = false)] show_url: bool,
  #[prop(default="Copy clone URL".into())] label: String,
  #[prop(default = false)] primary: bool,
) -> impl IntoView {
  let leptos_use_clipboard::UseClipboardReturn {
    copy,
    copied,
    is_supported,
    ..
  } = leptos_use_clipboard::use_clipboard();
  let code = value.clone();
  view! {
      {if show_url { Some(view! { <code>{code}</code> }) } else { None }}
      <thaw::Button
          appearance=if primary {
              thaw::ButtonAppearance::Primary
          } else {
              thaw::ButtonAppearance::Secondary
          }
          disabled=Signal::derive(move || !is_supported.get())
          on_click=move |_| copy(&value)
      >
          <thaw::Icon icon=icondata_lu::LuCopy width="14px" height="14px" />
          {move || crate::i18n::t(if copied.get() { "Copied" } else { &label })}
      </thaw::Button>
  }
}

#[component]
fn ProjectHeader(project: serde_json::Value, permissions: serde_json::Value) -> impl IntoView {
  let mark = api::text(&project, "key")
    .chars()
    .take(2)
    .collect::<String>()
    .to_uppercase();
  view! {
      <section class="project-header-card">
          <div class="project-header-identity">
              <span class="project-mark">{mark}</span>
              <div>
                  <div>
                      <h1>{api::text(&project, "name")}</h1>
                      <span class="badge mono">{api::text(&project, "visibility")}</span>
                  </div>
                  <p class="mono">{api::text(&project, "full_path")}</p>
                  <small>
                      {
                          let description = api::text(&project, "description");
                          if description.is_empty() {
                              crate::i18n::t("No description provided.")
                          } else {
                              description
                          }
                      }
                  </small>
              </div>
          </div>
          <div class="toolbar">
              <span class="badge outline">
                  <thaw::Icon icon=icondata_lu::LuGitBranch width="14px" height="14px" />
                  <span class="mono">{api::text(&project, "default_branch")}</span>
              </span>
              <span class="badge outline">
                  <thaw::Icon icon=icondata_lu::LuShieldCheck width="14px" height="14px" />
                  {if permissions["role_level"].as_u64().is_some_and(|level| level >= 100) {
                      "super-admin".into()
                  } else {
                      api::text(&permissions, "role")
                  }}
              </span>
              <A href="/app/projects" {..} class="button">
                  <thaw::Icon icon=icondata_lu::LuArrowLeft width="14px" height="14px" />
                  {crate::i18n::t("Projects")}
              </A>
              <CloneUrl
                  value=api::text(&project, "clone_http_url")
                  label="Clone".into()
                  primary=true
              />
          </div>
      </section>
  }
}
