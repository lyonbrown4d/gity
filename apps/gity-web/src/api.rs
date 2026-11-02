use futures::{
  FutureExt,
  future::{LocalBoxFuture, Shared},
};
use gloo_net::http::{Method, RequestBuilder};
use serde_json::{Value, json};
use std::cell::RefCell;
mod navigation;
mod pagination;
pub(crate) use navigation::safe_after_login_redirect;
pub use pagination::list_all_pages;
type RefreshFuture = Shared<LocalBoxFuture<'static, Result<(), String>>>;
thread_local! { static REFRESH: RefCell<Option<RefreshFuture>> = const { RefCell::new(None) }; }

pub fn storage(key: &str) -> String {
  web_sys::window()
    .and_then(|w| w.local_storage().ok().flatten())
    .and_then(|s| s.get_item(key).ok().flatten())
    .unwrap_or_default()
}
pub fn store(key: &str, value: &str) {
  if let Some(s) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
    let _ = s.set_item(key, value);
  }
}
pub fn clear_auth() {
  store("gity.access_token", "");
  store("gity.refresh_token", "");
}
pub fn save_auth(value: &Value) {
  store("gity.access_token", &text(value, "token"));
  store("gity.refresh_token", &text(value, "refresh_token"));
}
pub fn text(value: &Value, key: &str) -> String {
  let v = value.get(key).unwrap_or(&Value::Null);
  match v {
    Value::String(s) => s.clone(),
    Value::Null => String::new(),
    _ => v.to_string(),
  }
}
pub fn list(value: &Value) -> Vec<Value> {
  if let Some(rows) = value.as_array() {
    return rows.clone();
  }
  for key in [
    "items",
    "data",
    "records",
    "results",
    "pages",
    "packages",
    "locks",
    "objects",
    "runners",
    "jobs",
    "pipelines",
    "releases",
    "comments",
    "members",
    "approvals",
    "rules",
  ] {
    if let Some(rows) = value.get(key).and_then(Value::as_array) {
      return rows.clone();
    }
  }
  Vec::new()
}
pub fn enc(value: &str) -> String {
  js_sys::encode_uri_component(value)
    .as_string()
    .unwrap_or_default()
}
pub fn navigate(path: &str) {
  if let Some(w) = web_sys::window() {
    let _ = w.location().set_href(path);
  }
}
async fn once(
  method: &str,
  path: &str,
  body: Option<Value>,
  auth: bool,
) -> Result<(u16, Value), String> {
  let method = match method {
    "POST" => Method::POST,
    "PATCH" => Method::PATCH,
    "PUT" => Method::PUT,
    "DELETE" => Method::DELETE,
    _ => Method::GET,
  };
  let mut req = RequestBuilder::new(&format!("/api/v1{path}"))
    .method(method)
    .header("Accept-Language", &crate::i18n::locale());
  if auth {
    req = req.header(
      "Authorization",
      &format!("Bearer {}", storage("gity.access_token")),
    );
  }
  let response = if let Some(body) = body {
    req
      .header("Content-Type", "application/json")
      .body(body.to_string())
      .map_err(|e| e.to_string())?
      .send()
      .await
  } else {
    req.send().await
  }
  .map_err(|e| e.to_string())?;
  let status = response.status();
  let raw = response.text().await.map_err(|e| e.to_string())?;
  let data = serde_json::from_str(&raw).unwrap_or({
    if raw.is_empty() {
      Value::Null
    } else {
      Value::String(raw)
    }
  });
  Ok((status, data))
}
pub async fn request(method: &str, path: &str, body: Option<Value>) -> Result<Value, String> {
  let auth = !path.starts_with("/auth/login") && !path.starts_with("/auth/refresh");
  let (mut status, mut data) = once(method, path, body.clone(), auth).await?;
  if status == 401 && auth {
    let refresh = REFRESH.with(|slot| {
      let mut slot = slot.borrow_mut();
      slot
        .get_or_insert_with(|| {
          async {
            let refresh = storage("gity.refresh_token");
            let (status, value) = once(
              "POST",
              "/auth/refresh",
              Some(json!({"refresh_token":refresh})),
              false,
            )
            .await?;
            if (200..300).contains(&status) {
              save_auth(value.get("data").unwrap_or(&value));
              Ok(())
            } else {
              clear_auth();
              let redirect = web_sys::window()
                .and_then(|window| {
                  let location = window.location();
                  safe_after_login_redirect(&format!(
                    "{}{}",
                    location.pathname().ok()?,
                    location.search().ok()?
                  ))
                })
                .unwrap_or_else(|| "/login".into());
              navigate(&redirect);
              Err("Your session expired. Please sign in again.".into())
            }
          }
          .boxed_local()
          .shared()
        })
        .clone()
    });
    let refreshed = refresh.await;
    REFRESH.with(|slot| *slot.borrow_mut() = None);
    refreshed?;
    (status, data) = once(method, path, body, auth).await?;
  }
  if !(200..300).contains(&status) {
    let message = text(&data, "message");
    return Err(if message.is_empty() {
      format!("Request failed ({status}): {data}")
    } else {
      message
    });
  }
  Ok(data.get("data").cloned().unwrap_or(data))
}

#[cfg(test)]
#[path = "tests/api_contracts.rs"]
mod tests;
