use super::Context;
use crate::{api, i18n::t};
use leptos::prelude::*;

#[component]
pub(super) fn AutomationGuide(context: Context) -> impl IntoView {
  view! {
      <section class="settings-automation">
          <div class="settings-section-header">
              <div>
                  <h3>{t("Clone and automation guide")}</h3>
                  <p>
                      {t(
                          "Use the HTTP clone URL with a project access token or deploy token. Tokens are sent as Git Basic auth passwords.",
                      )}
                  </p>
              </div>
              <div class="settings-scopes">
                  <span class="badge">"HTTP"</span>
                  <span class="badge outline">{t("Project access tokens")}</span>
                  <span class="badge outline">{t("Deploy tokens")}</span>
              </div>
          </div>
          {move || {
              view! {
                  <Command
                      label="HTTP clone command"
                      value=format!(
                          "git clone {}",
                          api::text(&context.project.data.get(), "clone_http_url"),
                      )
                  />
              }
          }}
          <div class="settings-guide-grid">
              <div>
                  <h4>{t("Project access token scopes")}</h4>
                  {[
                      ("read_repository", "Clone and fetch repository contents."),
                      ("write_repository", "Push commits and branches."),
                      ("read_package", "Download package files and install dependencies."),
                      ("write_package", "Publish or upload package files."),
                      ("read_api", "Read project API data for automation."),
                      ("write_api", "Write project API data for automation."),
                  ]
                      .into_iter()
                      .map(|(scope, text)| {
                          view! {
                              <div class="settings-scope-guide">
                                  <span class="badge outline">{scope}</span>
                                  <small>{t(text)}</small>
                              </div>
                          }
                      })
                      .collect_view()}
                  <small>
                      {t(
                          "Project access tokens can also use read_api/write_api for issue, merge request, wiki, job and runner automation.",
                      )}
                  </small>
              </div>
              <div>
                  <h4>{t("Deploy token fit")}</h4>
                  <p>
                      {t(
                          "Deploy tokens are best for CI/CD and production deploys that only need repository or package access.",
                      )}
                  </p>
                  <small>
                      {t("Use read_repository for clone/fetch; add write_repository for push.")}
                  </small>
                  <small>
                      {t("Use read_package/write_package for package pull/publish automation.")}
                  </small>
              </div>
              <div>
                  <h4>{t("Deploy key status")}</h4>
                  <p>
                      {t(
                          "Deploy keys authenticate SSH Git access on the configured SSH port. Read-only keys can clone and pull; can-push keys can also push within project branch protection.",
                      )}
                  </p>
              </div>
          </div>
          <Command
              label="Package automation command"
              value=package_command(&context.path.get_value(), "<token>")
          />
      </section>
  }
}
#[component]
pub(super) fn CreatedToken(context: Context) -> impl IntoView {
  view! {
      <div class="settings-created-token" role="status">
          <strong>{t("Token created. Copy it now because it will not be shown again.")}</strong>
          <small>
              {t(
                  "When Git asks for credentials, enter the token username as the username and the token as the password.",
              )}
          </small>
          {move || {
              let created = context.created.get();
              let token = api::text(&created, "token");
              let username = api::text(&created["project_token"], "username");
              let clone = api::text(&context.project.data.get(), "clone_http_url");
              let authenticated = authenticated_url(&clone, &username, &token);
              view! {
                  <div class="settings-created-grid">
                      <Command label="Token username" value=username />
                      <Command label="Token" value=token.clone() />
                      <Command
                          label="Git clone with token"
                          value=format!("git clone {}", shell_quote(&authenticated))
                      />
                      <Command
                          label="Git remote with token"
                          value=format!(
                              "git remote set-url origin {}",
                              shell_quote(&authenticated),
                          )
                      />
                  </div>
                  <Command
                      label="Package API with token"
                      value=package_command(&context.path.get_value(), &token)
                  />
              }
          }}
          <small>
              {t(
                  "Store tokens in CI secrets or local credential helpers instead of committing them to repository files.",
              )}
          </small>
          <button type="button" on:click=move |_| context.created.set(serde_json::Value::Null)>
              {t("Dismiss")}
          </button>
      </div>
  }
}
#[component]
fn Command(label: &'static str, value: String) -> impl IntoView {
  let clipboard = leptos_use_clipboard::use_clipboard();
  let copy = clipboard.copy;
  let copied = clipboard.copied;
  let supported = clipboard.is_supported;
  let shown = value.clone();
  view! {
      <div class="settings-command">
          <div>
              <small>{t(label)}</small>
              <button
                  type="button"
                  disabled=move || !supported.get()
                  on:click=move |_| copy(&value)
              >
                  <thaw::Icon icon=icondata_lu::LuCopy width="14px" height="14px" />
                  {move || t(if copied.get() { "Copied" } else { "Copy command" })}
              </button>
          </div>
          <code>{shown}</code>
      </div>
  }
}
fn package_command(path: &str, token: &str) -> String {
  let origin = web_sys::window()
    .and_then(|w| w.location().origin().ok())
    .unwrap_or_default();
  format!(
    "curl --header {} {}",
    shell_quote(&format!("Authorization: Bearer {token}")),
    shell_quote(&format!("{origin}/api/v1{path}/packages"))
  )
}
fn authenticated_url(url: &str, username: &str, token: &str) -> String {
  let Ok(url) = web_sys::Url::new(url) else {
    return String::new();
  };
  if !["http:", "https:"].contains(&url.protocol().as_str()) {
    return String::new();
  }
  url.set_username(username);
  url.set_password(token);
  url.href()
}

fn shell_quote(value: &str) -> String {
  format!("'{}'", value.replace('\'', "'\\''"))
}
