use crate::api;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use leptos::prelude::*;
use serde_json::Value;
use wasm_bindgen::{JsCast, JsValue};
#[component]
pub fn Download(path: String, filename: String) -> impl IntoView {
  let busy = RwSignal::new(false);
  let error = RwSignal::new(String::new());
  let action = move |_| {
    if busy.get_untracked() {
      return;
    }
    let path = path.clone();
    let filename = filename.clone();
    busy.set(true);
    error.set(String::new());
    wasm_bindgen_futures::spawn_local(async move {
      let result = async {
        let v = api::request("GET", &path, None).await?;
        let bytes = STANDARD
          .decode(api::text(&v, "content_base64"))
          .map_err(|e| e.to_string())?;
        let buffer = js_sys::Uint8Array::from(bytes.as_slice());
        let chunks = js_sys::Array::new();
        chunks.push(&buffer);
        let blob = web_sys::Blob::new_with_u8_array_sequence(&chunks)
          .map_err(|_| "Cannot create download".to_string())?;
        let url = web_sys::Url::create_object_url_with_blob(&blob)
          .map_err(|_| "Cannot create download URL".to_string())?;
        let doc = web_sys::window()
          .and_then(|w| w.document())
          .ok_or("Document unavailable")?;
        let a: web_sys::HtmlAnchorElement = doc
          .create_element("a")
          .map_err(|_| "Cannot create download link")?
          .unchecked_into();
        a.set_href(&url);
        a.set_download(&filename);
        a.click();
        let _ = web_sys::Url::revoke_object_url(&url);
        Ok::<_, String>(())
      }
      .await;
      if busy.is_disposed() {
        return;
      }
      busy.set(false);
      if let Err(e) = result {
        error.set(e);
      }
    });
  };
  view! {
      <span class="action">
          <button type="button" disabled=move || busy.get() on:click=action>
              <thaw::Icon icon=icondata_lu::LuDownload width="16px" height="16px" />
              {crate::i18n::t("Download")}
          </button>
          <span class="error" role="alert">
              {move || error.get()}
          </span>
      </span>
  }
}
#[component]
pub fn IssueAttachment(
  path: String,
  #[prop(optional)] on_uploaded: Option<Callback<String>>,
) -> impl IntoView {
  let error = RwSignal::new(String::new());
  let markdown = RwSignal::new(String::new());
  let busy = RwSignal::new(false);
  let submit = move |ev: leptos::ev::SubmitEvent| {
    ev.prevent_default();
    if busy.get_untracked() {
      return;
    }
    let form: web_sys::HtmlFormElement = ev.target().unwrap().unchecked_into();
    let Ok(data) = web_sys::FormData::new_with_form(&form) else {
      return;
    };
    let path = path.clone();
    busy.set(true);
    error.set(String::new());
    wasm_bindgen_futures::spawn_local(async move {
      let result = async {
        let response = gloo_net::http::Request::post(&format!("/api/v1{path}"))
          .header("Accept-Language", &crate::i18n::locale())
          .header(
            "Authorization",
            &format!("Bearer {}", api::storage("gity.access_token")),
          )
          .body(JsValue::from(data))
          .map_err(|e| e.to_string())?
          .send()
          .await
          .map_err(|e| e.to_string())?;
        let status = response.status();
        let v: Value = response.json().await.map_err(|e| e.to_string())?;
        if !(200..300).contains(&status) {
          return Err(api::text(&v, "message"));
        }
        Ok(v.get("data").cloned().unwrap_or(v))
      }
      .await;
      if busy.is_disposed() {
        return;
      }
      busy.set(false);
      match result {
        Ok(v) => {
          let m = api::text(&v, "markdown");
          let uploaded = if m.is_empty() {
            format!("[{}]({})", api::text(&v, "file_name"), api::text(&v, "url"))
          } else {
            m
          };
          markdown.set(uploaded.clone());
          if let Some(callback) = on_uploaded {
            callback.run(uploaded);
          }
        }
        Err(e) => error.set(e),
      }
    });
  };
  view! {
      <details class="composer">
          <summary>{crate::i18n::t("Attach a file")}</summary>
          <form on:submit=submit>
              <label>{crate::i18n::t("File")}<input type="file" name="file" required /></label>
              <button disabled=move || busy.get()>{crate::i18n::t("Upload attachment")}</button>
          </form>
          <p class="error" role="alert">
              {move || error.get()}
          </p>
          <Show when=move || !markdown.get().is_empty()>
              <p class="muted">{crate::i18n::t("Paste this link into your comment:")}</p>
              <textarea readonly rows="2" prop:value=move || markdown.get() />
          </Show>
      </details>
  }
}
