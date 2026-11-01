use super::controller::OrganizationAdmin;
use crate::{app::admin_ui::ConsolePanel, i18n::t};
use leptos::prelude::*;

#[component]
pub(super) fn OrganizationPanels(state: OrganizationAdmin) -> impl IntoView {
  view! {
      <div class="admin-organization-panels">
          <MembersPanel state=state />
          <ProjectsPanel state=state />
      </div>
  }
}
#[component]
fn MembersPanel(state: OrganizationAdmin) -> impl IntoView {
  view! {
      <ConsolePanel
          title="Members"
          description="Members of the selected organization."
          meta=Signal::derive(move || format!("{} {}", state.member_rows.get().len(), t("total")))
      >
          <LoadingRows loading=state.loading />
          <Show when=move || !state.loading.get()>
              <div class="table-scroll">
                  <table class="admin-directory-table members-table">
                      <thead>
                          <tr>
                              <th>{t("Username")}</th>
                              <th>{t("Role")}</th>
                              <th>{t("User ID")}</th>
                          </tr>
                      </thead>
                      <tbody>
                          {move || {
                              state
                                  .member_rows
                                  .get()
                                  .into_iter()
                                  .map(|member| {
                                      view! {
                                          <tr>
                                              <td>
                                                  <div class="admin-directory-identity">
                                                      <i class="active"></i>
                                                      <div>
                                                          <p>{member.username}</p>
                                                          <small>{member.email}</small>
                                                      </div>
                                                  </div>
                                              </td>
                                              <td>
                                                  <span class="badge">{member.role}</span>
                                              </td>
                                              <td class="mono user-id">{member.user_id}</td>
                                          </tr>
                                      }
                                  })
                                  .collect_view()
                          }}
                      </tbody>
                  </table>
              </div>
              <Show when=move || state.member_rows.get().is_empty()>
                  <Empty message="No members found." />
              </Show>
          </Show>
      </ConsolePanel>
  }
}
#[component]
fn ProjectsPanel(state: OrganizationAdmin) -> impl IntoView {
  view! {
      <ConsolePanel
          title="Projects"
          description="Projects under the selected organization."
          meta=Signal::derive(move || {
              format!("{} {}", state.project_rows.get().len(), t("projects"))
          })
      >
          <LoadingRows loading=state.loading />
          <Show when=move || !state.loading.get()>
              <div class="table-scroll">
                  <table class="admin-directory-table organization-projects-table">
                      <thead>
                          <tr>
                              <th>{t("Project")}</th>
                              <th>{t("Visibility")}</th>
                              <th>{t("Default branch")}</th>
                          </tr>
                      </thead>
                      <tbody>
                          {move || {
                              state
                                  .project_rows
                                  .get()
                                  .into_iter()
                                  .map(|project| {
                                      view! {
                                          <tr>
                                              <td>
                                                  <p class="admin-directory-name">{project.name}</p>
                                                  <small class="mono">{project.key}</small>
                                              </td>
                                              <td>
                                                  <span class="badge outline">{project.visibility}</span>
                                              </td>
                                              <td class="mono">{project.default_branch}</td>
                                          </tr>
                                      }
                                  })
                                  .collect_view()
                          }}
                      </tbody>
                  </table>
              </div>
              <Show when=move || state.project_rows.get().is_empty()>
                  <Empty message="No projects found." />
              </Show>
          </Show>
      </ConsolePanel>
  }
}
#[component]
pub(crate) fn LoadingRows(loading: Signal<bool>) -> impl IntoView {
  view! {
      <Show when=move || loading.get()>
          <div class="console-loading" aria-label=t("Loading")>
              {(0..4).map(|_| view! { <span class="loading-placeholder"></span> }).collect_view()}
          </div>
      </Show>
  }
}
#[component]
fn Empty(message: &'static str) -> impl IntoView {
  view! {
      <div class="admin-directory-empty">
          <thaw::Icon icon=icondata_lu::LuCircleDotDashed width="20px" height="20px" />
          <p>{t(message)}</p>
      </div>
  }
}
