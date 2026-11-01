use crate::{api, i18n::t, ui::query};
use leptos::prelude::*;
use serde_json::json;

#[component]
pub(super) fn Profile() -> impl IntoView {
  let q = query("/users/me".into());
  let username = RwSignal::new(String::new());
  let email = RwSignal::new(String::new());
  let password = RwSignal::new(String::new());
  let busy = RwSignal::new(false);
  let message = RwSignal::new(String::new());
  let error = RwSignal::new(String::new());
  Effect::new(move |_| {
    let user = q.data.get();
    if user.is_object() {
      username.set(api::text(&user, "username"));
      email.set(api::text(&user, "email"));
    }
  });
  let submit = move |event: leptos::ev::SubmitEvent| {
    event.prevent_default();
    if busy.get_untracked() || q.loading.get_untracked() {
      return;
    }
    busy.set(true);
    error.set(String::new());
    message.set(String::new());
    let mut body = json!({"username":username.get_untracked(), "email":email.get_untracked()});
    let password_changed = !password.get_untracked().is_empty();
    if password_changed {
      body["password"] = json!(password.get_untracked());
    }
    wasm_bindgen_futures::spawn_local(async move {
      let result = api::request("PATCH", "/users/me", Some(body)).await;
      if busy.is_disposed() {
        return;
      }
      match result {
        Ok(_) => {
          password.set(String::new());
          if password_changed {
            api::clear_auth();
            api::navigate("/login");
            return;
          }
          message.set(t("Profile updated."));
          q.refresh.update(|n| *n += 1);
        }
        Err(cause) => error.set(cause),
      }
      busy.set(false);
    });
  };
  view! {
      <div class="profile-page page-enter">
          <header class="profile-page-header">
              <div>
                  <p>{t("My Account")}</p>
                  <h1>{t("Profile")}</h1>
                  <p>{t("Current account information and access status.")}</p>
              </div>
              <span class="badge" class:primary=move || q.data.get()["is_super_admin"] == true>
                  {move || t(
                      if q.data.get()["is_super_admin"] == true { "Super Admin" } else { "User" },
                  )}
              </span>
          </header>
          <Show when=move || !q.error.get().is_empty()>
              <p class="error" role="alert">
                  {move || q.error.get()}
              </p>
          </Show>
          <div class="profile-columns">
              <section class="account-card">
                  <header>
                      <h2>{t("My Account")}</h2>
                      <p>{t("Identity and quick navigation.")}</p>
                  </header>
                  <Show when=move || q.loading.get()>
                      <div class="profile-skeleton" aria-label=t("Loading…")>
                          {(0..4)
                              .map(|_| view! { <div class="loading-placeholder"></div> })
                              .collect_view()}
                      </div>
                  </Show>
                  <Show when=move || !q.loading.get()>
                      <dl class="profile-details">
                          <ProfileDetail
                              label="Username"
                              value=Signal::derive(move || api::text(&q.data.get(), "username"))
                          />
                          <ProfileDetail
                              label="Email"
                              value=Signal::derive(move || api::text(&q.data.get(), "email"))
                          />
                          <ProfileDetail
                              label="Account Status"
                              value=Signal::derive(move || api::text(&q.data.get(), "status"))
                          />
                          <ProfileDetail
                              label="Access role"
                              value=Signal::derive(move || t(
                                  if q.data.get()["is_super_admin"] == true {
                                      "Super Admin"
                                  } else {
                                      "User"
                                  },
                              ))
                          />
                      </dl>
                  </Show>
              </section>
              <section class="account-card">
                  <header>
                      <h2>{t("Edit Profile")}</h2>
                      <p>{t("Update username, email, and optionally change your password.")}</p>
                  </header>
                  <div class="profile-editor">
                      <form on:submit=submit>
                          <label for="profile-username">
                              {t("Username")}
                              <input
                                  id="profile-username"
                                  autocomplete="username"
                                  prop:value=move || username.get()
                                  on:input=move |event| username.set(event_target_value(&event))
                              />
                          </label>
                          <label for="profile-email">
                              {t("Email")}
                              <input
                                  id="profile-email"
                                  type="email"
                                  autocomplete="email"
                                  prop:value=move || email.get()
                                  on:input=move |event| email.set(event_target_value(&event))
                              />
                          </label>
                          <label class="wide" for="profile-password">
                              {t("New Password (Optional)")}
                              <input
                                  id="profile-password"
                                  type="password"
                                  autocomplete="new-password"
                                  prop:value=move || password.get()
                                  on:input=move |event| password.set(event_target_value(&event))
                              />
                          </label>
                          <footer>
                              <p>{t("Current account information and access status.")}</p>
                              <button
                                  class="primary"
                                  type="submit"
                                  disabled=move || busy.get() || q.loading.get()
                              >
                                  {move || {
                                      if busy.get() {
                                          format!("{}...", t("Save"))
                                      } else {
                                          t("Save Changes")
                                      }
                                  }}
                              </button>
                          </footer>
                      </form>
                      <div class="profile-feedback" aria-live="polite">
                          <Show when=move || !message.get().is_empty()>
                              <p class="success" role="status">
                                  {move || message.get()}
                              </p>
                          </Show>
                          <Show when=move || !error.get().is_empty()>
                              <p class="error" role="alert">
                                  {move || error.get()}
                              </p>
                          </Show>
                      </div>
                  </div>
              </section>
          </div>
      </div>
  }
}

#[component]
fn ProfileDetail(label: &'static str, value: Signal<String>) -> impl IntoView {
  let icon = match label {
    "Username" => icondata_lu::LuUserRound,
    "Email" => icondata_lu::LuAtSign,
    "Account Status" => icondata_lu::LuShieldCheck,
    _ => icondata_lu::LuKeyRound,
  };
  view! {
      <div>
          <thaw::Icon icon=icon width="16px" height="16px" />
          <dt>{t(label)}</dt>
          <dd title=move || {
              value.get()
          }>
              {move || {
                  let text = value.get();
                  if text.is_empty() { "--".into() } else { text }
              }}
          </dd>
      </div>
  }
}
