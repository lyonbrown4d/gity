use crate::api;
use leptos::prelude::*;
use serde_json::json;
use wasm_bindgen::JsCast;
#[component]
pub(super) fn Login() -> impl IntoView {
  let busy = RwSignal::new(false);
  let error = RwSignal::new(String::new());
  let submit = move |ev: leptos::ev::SubmitEvent| {
    ev.prevent_default();
    if busy.get_untracked() {
      return;
    }
    let form: web_sys::HtmlFormElement = ev.target().unwrap().unchecked_into();
    let Ok(data) = web_sys::FormData::new_with_form(&form) else {
      return;
    };
    let username = data.get("username").as_string().unwrap_or_default();
    let password = data.get("password").as_string().unwrap_or_default();
    busy.set(true);
    error.set(String::new());
    wasm_bindgen_futures::spawn_local(async move {
      match api::request(
        "POST",
        "/auth/login",
        Some(json!({"username":username,"password":password})),
      )
      .await
      {
        Ok(v) => {
          api::save_auth(&v);
          api::store("gity.user_id", &api::text(&v, "user_id"));
          api::navigate("/app/dashboard");
        }
        Err(e) => error.set(e),
      }
      busy.set(false);
    });
  };
  view! {
      <main class="login-page">
          <div class="login-view-controls">
              <super::controls::ViewControls />
          </div>
          <div class="login-card">
              <aside class="login-story">
                  <div>
                      <div class="login-brand">
                          <span>
                              <thaw::Icon icon=icondata_lu::LuBraces width="16px" height="16px" />
                          </span>
                          <div>
                              <strong>{crate::i18n::t("Gity")}</strong>
                              <small>{crate::i18n::t("Code Canvas")}</small>
                          </div>
                      </div>
                      <div class="login-intro">
                          <p>{crate::i18n::t("Workspace")}</p>
                          <h2>
                              {crate::i18n::t(
                                  "Project activity, organization scope, and account access in one workspace.",
                              )}
                          </h2>
                      </div>
                  </div>
                  <div class="login-features">
                      <div>
                          <span>
                              <thaw::Icon
                                  icon=icondata_lu::LuGitBranch
                                  width="16px"
                                  height="16px"
                              />
                          </span>
                          <strong>{crate::i18n::t("Projects")}</strong>
                          <small>{crate::i18n::t("Code")}</small>
                      </div>
                      <div>
                          <span>
                              <thaw::Icon
                                  icon=icondata_lu::LuWorkflow
                                  width="16px"
                                  height="16px"
                              />
                          </span>
                          <strong>{crate::i18n::t("CI/CD")}</strong>
                          <small>{crate::i18n::t("Operate")}</small>
                      </div>
                      <div>
                          <span>
                              <thaw::Icon
                                  icon=icondata_lu::LuLockKeyhole
                                  width="16px"
                                  height="16px"
                              />
                          </span>
                          <strong>{crate::i18n::t("Access role")}</strong>
                          <small>{crate::i18n::t("Account Status")}</small>
                      </div>
                  </div>
              </aside>
              <form class="login-form" on:submit=submit>
                  <div class="login-fields">
                      <header>
                          <p class="muted">{crate::i18n::t("User Workspace")}</p>
                          <h1>{crate::i18n::t("Welcome back")}</h1>
                          <p class="muted">
                              {crate::i18n::t("Sign in to manage projects and organizations.")}
                          </p>
                      </header>
                      <label for="login-username">
                          {crate::i18n::t("Username / Email")}
                          <input
                              id="login-username"
                              name="username"
                              placeholder="admin@example.com"
                              autocomplete="username"
                              required
                              autofocus
                          />
                      </label>
                      <label for="login-password">
                          {crate::i18n::t("Password")}
                          <input
                              id="login-password"
                              name="password"
                              type="password"
                              autocomplete="current-password"
                              required
                          />
                      </label>
                      <Show when=move || !error.get().is_empty()>
                          <p role="alert" class="error">
                              {move || error.get()}
                          </p>
                      </Show>
                      <thaw::Button
                          appearance=thaw::ButtonAppearance::Primary
                          button_type=thaw::ButtonType::Submit
                          loading=Signal::derive(move || busy.get())
                          disabled=Signal::derive(move || busy.get())
                      >
                          {crate::i18n::t("Sign in")}
                      </thaw::Button>
                  </div>
              </form>
          </div>
      </main>
  }
}
