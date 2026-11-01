use super::admin_overview::AdminOverview;
use super::admin_users::AdminUsers;
use super::dashboard::Dashboard;
use super::project::Project;
use super::projects::Projects;
use super::{admin_orgs::AdminOrganizations, admin_projects::AdminProjects};
use super::{profile::Profile, todos::Todos};
use crate::ui::*;
use leptos::prelude::*;
use serde_json::Value;
#[component]
pub(super) fn Page(path: String, search: String) -> impl IntoView {
  let parts = path.trim_matches('/').split('/').collect::<Vec<_>>();
  let params = web_sys::UrlSearchParams::new_with_str(&search).ok();
  let tab = params
    .as_ref()
    .and_then(|p| p.get("tab"))
    .unwrap_or_else(|| "overview".into());
  if parts.len() >= 4 && parts[0] == "app" && (parts[1] == "projects" || parts[1] == "repositories")
  {
    let id = parts[3].to_string();
    let base = format!("/app/projects/{}/{}", parts[2], id);
    let api_path = format!("/projects/{id}");
    if parts.len() >= 6 && parts[4] == "issues" {
      return view! { <crate::issues::IssueDetail project_path=api_path number=parts[5].to_string() base=base /> }.into_any();
    }
    return view! { <Project project_path=api_path base=base tab=tab /> }.into_any();
  }
  match path.as_str() {
    "/app/projects" | "/app/repositories" => view! { <Projects /> }.into_any(),
    "/app/profile" => view! { <Profile /> }.into_any(),
    "/app/todos" => view! { <Todos search=search /> }.into_any(),
    "/admin/orgs" => view! { <AdminOrganizations /> }.into_any(),
    "/admin/projects" | "/admin/repos" => view! { <AdminProjects /> }.into_any(),
    "/admin/users" => view! { <AdminUsers /> }.into_any(),
    "/admin" => view! { <AdminOverview /> }.into_any(),
    _ => view! { <Dashboard /> }.into_any(),
  }
}
#[component]
pub(super) fn AdminPage(path: String, search: String, me: Query) -> impl IntoView {
  view! {
      {move || {
          if me.loading.get() {
              view! { <p>{crate::i18n::t("Loading permissions…")}</p> }.into_any()
          } else if me.data.get().get("is_super_admin").and_then(Value::as_bool).unwrap_or(false)
          {
              view! { <Page path=path.clone() search=search.clone() /> }.into_any()
          } else {
              view! {
                  <p class="error" role="alert">
                      {crate::i18n::t("Administrator access is required.")}
                  </p>
              }
                  .into_any()
          }
      }}
  }
}
