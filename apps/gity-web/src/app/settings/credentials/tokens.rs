use super::{Context, SectionHeader};
use crate::app::settings::model;
use crate::{api, i18n::t, ui::*};
use leptos::prelude::*;
use serde_json::Value;

#[component]
pub(super) fn Tokens(context: Context, deploy: bool) -> impl IntoView {
  let query = if deploy {
    context.deploy
  } else {
    context.access
  };
  let resource = if deploy {
    "deploy-tokens"
  } else {
    "access-tokens"
  };
  let title = if deploy {
    "Deploy tokens"
  } else {
    "Project access tokens"
  };
  let description = if deploy {
    "Use deploy tokens for Git and package pull/push flows without granting full user access."
  } else {
    "Use project tokens for API, repository and package automation scoped to this project."
  };
  view! {
      <section class="settings-credential-section">
          <SectionHeader title=title description=description query=query />
          <TokenForm context=context deploy=deploy />
          <Status q=query />
          <div class="settings-credential-list">
              <Show when=move || !query.loading.get() && api::list(&query.data.get()).is_empty()>
                  <p>
                      {t(
                          if deploy {
                              "No deploy tokens yet."
                          } else {
                              "No project access tokens yet."
                          },
                      )}
                  </p>
              </Show>
              {move || {
                  api::list(&query.data.get())
                      .into_iter()
                      .map(|token| {
                          let active = token["active"] == true;
                          let path = format!(
                              "{}/{resource}/{}",
                              context.path.get_value(),
                              api::enc(&api::text(&token, "id")),
                          );
                          view! {
                              <div class="settings-credential-row">
                                  <div>
                                      <div class="settings-inline">
                                          <strong>{api::text(&token, "name")}</strong>
                                          <span class="badge" class:outline=!active>
                                              {t(if active { "active" } else { "revoked" })}
                                          </span>
                                      </div>
                                      <small>
                                          {format!("@{}", api::text(&token, "username"))}
                                      </small>
                                      <div class="settings-scopes">
                                          {api::list(&token["scopes"])
                                              .into_iter()
                                              .map(|scope| {
                                                  view! {
                                                      <span class="badge outline">
                                                          {t(scope.as_str().unwrap_or_default())}
                                                      </span>
                                                  }
                                              })
                                              .collect_view()}
                                      </div>
                                      <small>
                                          {format!(
                                              "{}: {} · {}: {}",
                                              t("Expires at"),
                                              date(&token, "expires_at"),
                                              t("Last used"),
                                              date(&token, "last_used_at"),
                                          )}
                                      </small>
                                  </div>
                                  <Action
                                      label="Revoke".into()
                                      path=path
                                      method="DELETE".into()
                                      refresh=query.refresh
                                      busy_state=context.mutation.busy
                                      confirm=true
                                      disabled=Signal::derive(move || {
                                          context.disabled.get() || !active
                                      })
                                      confirmation_title=t("Revoke token?")
                                      confirmation_description=t(
                                          "Existing automation using this token will stop working immediately.",
                                      )
                                      confirmation_label=t("Revoke")
                                  />
                              </div>
                          }
                      })
                      .collect_view()
              }}
          </div>
      </section>
  }
}
fn date(value: &Value, key: &str) -> String {
  let value = api::text(value, key);
  if value.is_empty() { t("Never") } else { value }
}
#[component]
fn TokenForm(context: Context, deploy: bool) -> impl IntoView {
  let name = RwSignal::new(String::new());
  let username = RwSignal::new(String::new());
  let expires = RwSignal::new(String::new());
  let scopes = RwSignal::new(vec!["read_repository".to_string()]);
  let prefix = if deploy {
    "deploy-token"
  } else {
    "project-access-token"
  };
  let submit = move |e: leptos::ev::SubmitEvent| {
    e.prevent_default();
    if context.disabled.get_untracked() {
      return;
    }
    match model::token_body(
      name.get_untracked(),
      username.get_untracked(),
      expires.get_untracked(),
      scopes.get_untracked(),
    ) {
      Ok(body) => context.mutation.run_with_result(
        "POST",
        format!(
          "{}/{}",
          context.path.get_value(),
          if deploy {
            "deploy-tokens"
          } else {
            "access-tokens"
          }
        ),
        body,
        Some(Callback::new(move |value: Value| {
          context.created.set(value);
          name.set(String::new());
          username.set(String::new());
          expires.set(String::new());
          scopes.set(vec!["read_repository".into()]);
        })),
      ),
      Err(error) => context.mutation.error.set(t(error)),
    }
  };
  view! {
      <form class="settings-token-form" on:submit=submit>
          <label for=format!(
              "{prefix}-name",
          )>
              {t("Name")}
              <input
                  id=format!("{prefix}-name")
                  prop:value=move || name.get()
                  on:input=move |e| name.set(event_target_value(&e))
                  placeholder=t(if deploy { "Production deploy" } else { "Release bot" })
                  disabled=move || context.disabled.get()
              />
          </label>
          {deploy
              .then(|| {
                  view! {
                      <label for="deploy-token-username">
                          {t("Username")}
                          <input
                              id="deploy-token-username"
                              prop:value=move || username.get()
                              on:input=move |e| username.set(event_target_value(&e))
                              placeholder=t("deploy-production")
                              disabled=move || context.disabled.get()
                          />
                      </label>
                  }
              })}
          <label for=format!(
              "{prefix}-expires",
          )>
              {t("Expires at")}
              <input
                  id=format!("{prefix}-expires")
                  type="date"
                  prop:value=move || expires.get()
                  on:input=move |e| expires.set(event_target_value(&e))
                  disabled=move || context.disabled.get()
              />
          </label>
          <button
              type="submit"
              class="primary"
              disabled=move || {
                  context.disabled.get() || name.get().trim().is_empty()
                      || scopes.get().is_empty()
              }
          >
              <thaw::Icon icon=icondata_lu::LuPlus width="16px" height="16px" />
              {t("Create token")}
          </button>
          <div class="settings-scopes settings-form-scopes">
              {model::SCOPES
                  .into_iter()
                  .take(if deploy { 4 } else { 6 })
                  .map(|scope| {
                      view! {
                          <button
                              type="button"
                              class:selected=move || scopes.get().iter().any(|v| v == scope)
                              aria-pressed=move || {
                                  scopes.get().iter().any(|v| v == scope).to_string()
                              }
                              disabled=move || context.disabled.get()
                              on:click=move |_| {
                                  scopes
                                      .update(|values| {
                                          if values.iter().any(|v| v == scope) {
                                              values.retain(|v| v != scope);
                                          } else {
                                              values.push(scope.into());
                                          }
                                      })
                              }
                          >
                              {t(scope)}
                          </button>
                      }
                  })
                  .collect_view()}
          </div>
      </form>
  }
}
