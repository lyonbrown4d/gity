mod credentials;
mod members;
mod model;
use crate::{api, i18n::t, ui::*};
use leptos::prelude::*;
use serde_json::json;

#[component]
pub(super) fn Settings(project_path: String, project: Query, permissions: Query) -> impl IntoView {
  let deleted = RwSignal::new(0u64);
  Effect::new(move || {
    if deleted.get() > 0 {
      api::navigate("/app/projects");
    }
  });
  let can_delete =
    Signal::derive(move || allowed(&permissions.data.get(), "project.delete", "can_delete", 50));
  let member_path = project_path.clone();
  let credential_path = project_path.clone();
  view! {
      <div class="settings-stack">
          <section class="settings-card account-card card-enter">
              <header>
                  <h2>{t("Settings")}</h2>
                  <p>{t("Project metadata and danger zone.")}</p>
              </header>
              <div class="settings-body">
                  <dl class="settings-metadata">
                      <div>
                          <dt>{t("Project UUID")}":"</dt>
                          <dd class="mono">{move || api::text(&project.data.get(), "uuid")}</dd>
                      </div>
                      <div>
                          <dt>{t("Project key")}":"</dt>
                          <dd>{move || api::text(&project.data.get(), "key")}</dd>
                      </div>
                      <div>
                          <dt>{t("Visibility")}":"</dt>
                          <dd>{move || api::text(&project.data.get(), "visibility")}</dd>
                      </div>
                      <div>
                          <dt>{t("Default branch")}":"</dt>
                          <dd>{move || api::text(&project.data.get(), "default_branch")}</dd>
                      </div>
                  </dl>
                  <div class="settings-danger">
                      <strong>{t("Danger zone")}</strong>
                      <p>{t("Deleting a project is irreversible.")}</p>
                      <Show when=move || !can_delete.get()>
                          <p class="settings-notice">
                              {t("Only project owners can delete this project.")}
                          </p>
                      </Show>
                      {move || {
                          let data = project.data.get();
                          view! {
                              <Action
                                  label="Delete".into()
                                  path=project_path.clone()
                                  method="DELETE".into()
                                  body=json!({"confirmation":api::text(&data,"full_path")})
                                  refresh=deleted
                                  confirm=true
                                  disabled=Signal::derive(move || !can_delete.get())
                                  confirmation_title=t("Delete project \"{name}\"?")
                                      .replace("{name}", &api::text(&data, "name"))
                                  confirmation_description=t("This action cannot be undone.")
                                  confirmation_label=t("Delete")
                              />
                          }
                      }}
                  </div>
              </div>
          </section>
          <members::Members project_path=member_path permissions=permissions />
          <credentials::Credentials
              project_path=credential_path
              project=project
              permissions=permissions
          />
      </div>
  }
}
#[component]
fn Header(title: &'static str, description: &'static str) -> impl IntoView {
  view! {
      <header>
          <h2>
              {match title {
                  "Project members" => {
                      Some(
                          view! {
                              <thaw::Icon
                                  icon=icondata_lu::LuUsersRound
                                  width="16px"
                                  height="16px"
                              />
                          }
                              .into_any(),
                      )
                  }
                  "Project credentials" => {
                      Some(
                          view! {
                              <thaw::Icon
                                  icon=icondata_lu::LuShieldCheck
                                  width="16px"
                                  height="16px"
                              />
                          }
                              .into_any(),
                      )
                  }
                  _ => None,
              }} {t(title)}
          </h2>
          <p>{t(description)}</p>
      </header>
  }
}
#[component]
fn Stat(label: &'static str, value: Signal<usize>) -> impl IntoView {
  view! {
      <div class="settings-stat">
          <small>{t(label)}</small>
          <strong>{move || value.get()}</strong>
      </div>
  }
}
