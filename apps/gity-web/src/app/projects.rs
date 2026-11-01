use super::project::CloneUrl;
use crate::{api, catalog, ui::*};
use leptos::prelude::*;
use leptos_router::components::A;
use serde_json::{Value, json};
#[component]
pub(super) fn Projects() -> impl IntoView {
  let orgs = query("/orgs".into());
  let selected = RwSignal::new(String::new());
  let open = RwSignal::new(false);
  Effect::new(move |_| {
    let data = orgs.data.get();
    if selected.get_untracked().is_empty()
      && let Some(first) = api::list(&data).first()
    {
      selected.set(api::text(first, "id"));
    }
  });
  let projects = query_fn(
    move || {
      if selected.get().is_empty() {
        String::new()
      } else {
        format!(
          "/projects?organization_id={}&page_size=200",
          api::enc(&selected.get())
        )
      }
    },
    true,
  );
  let org = Signal::derive(move || {
    api::list(&orgs.data.get())
      .into_iter()
      .find(|o| api::text(o, "id") == selected.get())
      .unwrap_or(Value::Null)
  });
  view! {
      <div class="project-directory-page">
          <section class="dashboard-summary">
              <div class="dashboard-summary-header">
                  <div>
                      <p class="workspace-path">{crate::i18n::t("Workspace / My Projects")}</p>
                      <h1>{crate::i18n::t("Projects")}</h1>
                      <p class="dashboard-description">
                          {crate::i18n::t(
                              "Browse organization namespaces, open project repositories, and copy clone URLs from one workspace.",
                          )}
                      </p>
                  </div>
                  <thaw::Button
                      appearance=thaw::ButtonAppearance::Primary
                      on_click=move |_| open.set(true)
                  >
                      {crate::i18n::t("＋ New Project")}
                  </thaw::Button>
              </div>
              <div class="scope-status">
                  <Scope
                      label="Current scope"
                      value=Signal::derive(move || api::text(&org.get(), "name"))
                  />
                  <Scope
                      label="Namespace key"
                      value=Signal::derive(move || api::text(&org.get(), "key"))
                  />
                  <Scope
                      label="Available projects"
                      value=Signal::derive(move || {
                          api::list(&projects.data.get()).len().to_string()
                      })
                  />
              </div>
          </section>
          <Status q=orgs />
          <Status q=projects />
          <div class="project-directory-columns">
              <section class="panel organization-scope">
                  <div class="panel-heading">
                      <div>
                          <h2>{crate::i18n::t("Organization")}</h2>
                          <p class="muted">
                              {crate::i18n::t("Choose the namespace that owns the projects.")}
                          </p>
                      </div>
                  </div>
                  <div class="scope-content">
                      <label for="organization-select">
                          {crate::i18n::t("Organization")}
                          <select
                              id="organization-select"
                              on:change=move |e| selected.set(event_target_value(&e))
                          >
                              {move || {
                                  api::list(&orgs.data.get())
                                      .into_iter()
                                      .map(|o| {
                                          let id = api::text(&o, "id");
                                          let chosen = id == selected.get();
                                          view! {
                                              <option value=id selected=chosen>
                                                  {api::text(&o, "name")}
                                              </option>
                                          }
                                      })
                                      .collect_view()
                              }}
                          </select>
                      </label>
                      <dl>
                          <div>
                              <dt>{crate::i18n::t("Namespace key")}</dt>
                              <dd>{move || api::text(&org.get(), "key")}</dd>
                          </div>
                          <div>
                              <dt>{crate::i18n::t("Available projects")}</dt>
                              <dd>{move || api::list(&projects.data.get()).len()}</dd>
                          </div>
                      </dl>
                      <thaw::Button
                          appearance=thaw::ButtonAppearance::Primary
                          block=true
                          on_click=move |_| open.set(true)
                      >
                          {crate::i18n::t("＋ New Project")}
                      </thaw::Button>
                  </div>
              </section>
              <section class="panel">
                  <div class="panel-heading">
                      <div>
                          <h2>{crate::i18n::t("Project directory")}</h2>
                          <p class="muted">
                              {move || {
                                  format!(
                                      "{} / {}",
                                      api::text(&org.get(), "name"),
                                      api::text(&org.get(), "key"),
                                  )
                              }}
                          </p>
                      </div>
                      <thaw::Button on_click=move |_| {
                          open.set(true)
                      }>{crate::i18n::t("＋ New Project")}</thaw::Button>
                  </div>
                  <Show when=move || {
                      projects.loading.get() && projects.data.get().is_null()
                  }>
                      <SkeletonRows rows=5 />
                  </Show>
                  {move || {
                      api::list(&projects.data.get())
                          .into_iter()
                          .map(|p| view! { <DirectoryRow project=p refresh=projects.refresh /> })
                          .collect_view()
                  }}
                  <Show when=move || {
                      !projects.loading.get() && api::list(&projects.data.get()).is_empty()
                  }>
                      <div class="empty">
                          <h3>{crate::i18n::t("No projects available.")}</h3>
                          <p>
                              {crate::i18n::t(
                                  "Create a project in this organization to get started.",
                              )}
                          </p>
                          <button on:click=move |_| {
                              open.set(true)
                          }>{crate::i18n::t("New Project")}</button>
                      </div>
                  </Show>
              </section>
          </div>
          <thaw::Dialog open=open>
              <thaw::DialogSurface>
                  <thaw::DialogBody>
                      <thaw::DialogTitle>{crate::i18n::t("Create Project")}</thaw::DialogTitle>
                      <thaw::DialogContent>
                          {move || {
                              view! {
                                  <EditorForm
                                      title="Project details".into()
                                      path="/projects".into()
                                      fields=catalog::project_fields()
                                      initial=json!({"organization_id":selected.get()})
                                      refresh=projects.refresh
                                      embedded=true
                                      on_saved=Callback::new(move |_| open.set(false))
                                  />
                              }
                          }}
                      </thaw::DialogContent>
                  </thaw::DialogBody>
              </thaw::DialogSurface>
          </thaw::Dialog>
      </div>
  }
}
#[component]
fn Scope(label: &'static str, value: Signal<String>) -> impl IntoView {
  view! {
      <div>
          <thaw::Icon icon=icondata_lu::LuFolderGit2 />
          <span>
              <small>{crate::i18n::t(label)}</small>
              <strong>{move || value.get()}</strong>
          </span>
      </div>
  }
}
#[component]
fn DirectoryRow(project: Value, refresh: RwSignal<u64>) -> impl IntoView {
  let path = format!(
    "/app/projects/{}/{}",
    api::text(&project, "organization_id"),
    api::text(&project, "id")
  );
  let name = api::text(&project, "name");
  view! {
      <article class="directory-row" aria-label=api::text(&project, "name")>
          <div class="directory-identity">
              <thaw::Icon icon=icondata_lu::LuFolderGit2 />
              <div>
                  <A href=path.clone()>{name}</A>
                  <span class="badge">{api::text(&project, "visibility")}</span>
                  <p>{api::text(&project, "full_path")}</p>
                  <p>
                      {
                          let text = api::text(&project, "description");
                          if text.is_empty() { "No description provided.".into() } else { text }
                      }
                  </p>
              </div>
          </div>
          <div class="directory-clone">
              <p>"⑂ "{api::text(&project, "default_branch")}</p>
              <CloneUrl
                  value=api::text(&project, "clone_http_url")
                  show_url=true
                  label="Copy".into()
              />
          </div>
          <div class="directory-actions">
              <A href={path} {..} class="button">
                  {crate::i18n::t("Open →")}
              </A>
              <Action
                  label="Delete".into()
                  path=format!("/projects/{}", api::text(&project, "id"))
                  method="DELETE".into()
                  body=json!({"confirmation":api::text(&project,"full_path")})
                  confirm=true
                  refresh=refresh
              />
          </div>
      </article>
  }
}
