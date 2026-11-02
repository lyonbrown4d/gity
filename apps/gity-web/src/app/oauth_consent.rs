use crate::api;
use leptos::prelude::*;
use serde::Deserialize;
use serde_json::json;

#[derive(Clone, Deserialize)]
struct PendingAuthorization {
  request_id: String,
  client_name: String,
  scopes: Vec<String>,
  resource: Option<String>,
  requests_admin_mode: bool,
}

fn request_id(search: &str) -> Option<String> {
  let params = web_sys::UrlSearchParams::new_with_str(search).ok()?;
  let id = params.get("request_id")?;
  if id.len() != 36
    || !id.bytes().enumerate().all(|(i, byte)| {
      if [8, 13, 18, 23].contains(&i) {
        byte == b'-'
      } else {
        byte.is_ascii_hexdigit()
      }
    })
  {
    return None;
  }
  Some(id)
}

fn valid_callback(value: &str) -> bool {
  if value.chars().any(char::is_control) {
    return false;
  }
  let Ok(url) = web_sys::Url::new(value) else {
    return false;
  };
  matches!(url.protocol().as_str(), "https:" | "http:") && url.username().is_empty()
}

#[component]
pub(super) fn OAuthConsent(search: String) -> impl IntoView {
  let id = request_id(&search);
  let pending = RwSignal::new(None::<PendingAuthorization>);
  let error = RwSignal::new(String::new());
  let busy = RwSignal::new(false);
  let admin_password = RwSignal::new(String::new());
  if let Some(id) = id.clone() {
    wasm_bindgen_futures::spawn_local(async move {
      match api::request("GET", &format!("/oauth/requests/{id}"), None).await {
        Ok(data) => match serde_json::from_value::<PendingAuthorization>(data) {
          Ok(details) if details.request_id == id => pending.set(Some(details)),
          Ok(_) => error.set(crate::i18n::t("Authorization request does not match.")),
          Err(_) => error.set(crate::i18n::t("Could not read authorization request.")),
        },
        Err(message) => error.set(message),
      }
    });
  } else {
    error.set(crate::i18n::t("Invalid authorization request."));
  }
  let decide = move |approve: bool| {
    if busy.get_untracked() {
      return;
    }
    let Some(details) = pending.get_untracked() else {
      return;
    };
    let password = admin_password.get_untracked();
    if approve && details.requests_admin_mode && password.is_empty() {
      error.set(crate::i18n::t(
        "Enter your password to approve administrator access.",
      ));
      return;
    }
    error.set(String::new());
    busy.set(true);
    wasm_bindgen_futures::spawn_local(async move {
      let mut body = json!({"approve": approve});
      if approve && details.requests_admin_mode {
        body["password"] = json!(password);
      }
      match api::request(
        "POST",
        &format!("/oauth/requests/{}", details.request_id),
        Some(body),
      )
      .await
      {
        Ok(result) => {
          let callback = api::text(&result, "redirect_uri");
          if valid_callback(&callback) {
            api::navigate(&callback);
          } else {
            error.set(crate::i18n::t("The authorization callback is invalid."));
          }
        }
        Err(message) => error.set(message),
      }
      busy.set(false);
    });
  };
  view! {
      <main class="login-page">
          <div class="login-view-controls"><super::controls::ViewControls /></div>
          <section class="login-card" aria-labelledby="oauth-consent-title">
              <div class="login-form">
                  <div class="login-fields">
                      <header>
                          <p class="muted">{crate::i18n::t("Gity authorization")}</p>
                          <h1 id="oauth-consent-title">{crate::i18n::t("Authorize application")}</h1>
                          <p class="muted">{crate::i18n::t("Review what this application will be able to access.")}</p>
                      </header>
                      {move || match pending.get() {
                        None => view! { <p class="muted">{crate::i18n::t("Loading authorization request…")}</p> }.into_any(),
                        Some(details) => {
                          let has_resource = details.resource.as_ref().is_some_and(|value| !value.is_empty());
                          view! {
                          <div class="oauth-consent-details">
                              <p><strong>{crate::i18n::t("Application")}</strong><br />{details.client_name.clone()}</p>
                              <p><strong>{crate::i18n::t("Requested access")}</strong></p>
                              <ul>{details.scopes.iter().map(|scope| view! { <li>{scope.clone()}</li> }).collect_view()}</ul>
                              <Show when=move || has_resource>
                                  <p><strong>{crate::i18n::t("Resource")}</strong><br />{details.resource.clone().unwrap_or_default()}</p>
                              </Show>
                              <Show when=move || details.requests_admin_mode>
                                  <p class="muted">{crate::i18n::t("This request asks to use administrator access.")}</p>
                                  <label for="oauth-admin-password">
                                      {crate::i18n::t("Confirm your password")}
                                      <input id="oauth-admin-password" type="password" autocomplete="current-password" prop:value=move || admin_password.get() on:input=move |ev| admin_password.set(event_target_value(&ev)) />
                                  </label>
                              </Show>
                          </div>
                        }.into_any()
                        }
                      }}
                      <Show when=move || !error.get().is_empty()>
                          <p role="alert" class="error">{move || error.get()}</p>
                      </Show>
                      <div class="oauth-consent-actions">
                          <thaw::Button appearance=thaw::ButtonAppearance::Primary button_type=thaw::ButtonType::Button disabled=Signal::derive(move || busy.get() || pending.get().is_none()) on:click=move |_| decide(true)>
                              {crate::i18n::t("Authorize")}
                          </thaw::Button>
                          <thaw::Button button_type=thaw::ButtonType::Button disabled=Signal::derive(move || busy.get() || pending.get().is_none()) on:click=move |_| decide(false)>
                              {crate::i18n::t("Deny")}
                          </thaw::Button>
                      </div>
                  </div>
              </div>
          </section>
      </main>
  }
}
