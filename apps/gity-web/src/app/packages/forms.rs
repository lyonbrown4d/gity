use super::model::UploadInput;
use crate::{i18n::t, ui::Mutation};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub(super) fn UploadForm(
  path: String,
  can_write: Signal<bool>,
  on_saved: Callback<()>,
) -> impl IntoView {
  let kind = RwSignal::new("generic".to_string());
  let name = RwSignal::new(String::new());
  let version = RwSignal::new("0.1.0".to_string());
  let filename = RwSignal::new("artifact.txt".to_string());
  let file_path = RwSignal::new(String::new());
  let content_type = RwSignal::new("text/plain".to_string());
  let inline = RwSignal::new(String::new());
  let encoded = RwSignal::new(String::new());
  let reading = RwSignal::new(false);
  let sequence = RwSignal::new(0u64);
  let mutation = Mutation::new(RwSignal::new(0));
  let select = move |event: leptos::ev::Event| {
    let input: web_sys::HtmlInputElement = event_target(&event);
    sequence.update(|v| *v += 1);
    let generation = sequence.get_untracked();
    encoded.set(String::new());
    let file = input
      .form()
      .and_then(|form| web_sys::FormData::new_with_form(&form).ok())
      .and_then(|data| data.get("file").dyn_into::<web_sys::File>().ok())
      .filter(|file| !file.name().is_empty());
    let Some(file) = file else {
      reading.set(false);
      return;
    };
    filename.set(file.name());
    content_type.set(if file.type_().is_empty() {
      "application/octet-stream".into()
    } else {
      file.type_()
    });
    if file_path.get_untracked().trim().is_empty() {
      file_path.set(file.name());
    }
    inline.set(String::new());
    reading.set(true);
    wasm_bindgen_futures::spawn_local(async move {
      let bytes = wasm_bindgen_futures::JsFuture::from(file.array_buffer()).await;
      if sequence.is_disposed() || sequence.get_untracked() != generation {
        return;
      }
      match bytes {
        Ok(buffer) => encoded.set(STANDARD.encode(js_sys::Uint8Array::new(&buffer).to_vec())),
        Err(_) => mutation.error.set(t("Cannot read file")),
      }
      reading.set(false);
    });
  };
  let submit = move |event: leptos::ev::SubmitEvent| {
    event.prevent_default();
    if mutation.busy.get_untracked() || reading.get_untracked() || !can_write.get_untracked() {
      return;
    }
    let body = UploadInput {
      kind: kind.get_untracked(),
      name: name.get_untracked(),
      version: version.get_untracked(),
      filename: filename.get_untracked(),
      path: file_path.get_untracked(),
      content_type: content_type.get_untracked(),
      inline: inline.get_untracked(),
      encoded: encoded.get_untracked(),
    }
    .body();
    match body {
      Ok(body) => mutation.run("POST", path.clone(), body, Some(on_saved)),
      Err(message) => mutation.error.set(t(message)),
    }
  };
  view! {
      <form class="packages-composer" on:submit=submit>
          <div class="packages-upload-identity">
              <label for="package-type">
                  {t("Package type")}
                  <select
                      id="package-type"
                      prop:value=move || kind.get()
                      on:change=move |e| kind.set(event_target_value(&e))
                  >
                      <option value="generic">"generic"</option>
                      <option value="npm">"npm"</option>
                      <option value="maven">"maven"</option>
                      <option value="container">"container"</option>
                  </select>
              </label>
              <Input id="package-name" label="Package name" value=name placeholder="gity-cli" />
              <Input id="package-version" label="Version" value=version placeholder="0.1.0" />
          </div>
          <div class="packages-upload-file">
              <Input
                  id="package-file-name"
                  label="File name"
                  value=filename
                  placeholder="artifact.tar.gz"
              />
              <Input
                  id="package-file-path"
                  label="File path optional"
                  value=file_path
                  placeholder="dist/artifact.tar.gz"
              />
              <Input
                  id="package-content-type"
                  label="Content type"
                  value=content_type
                  placeholder="application/octet-stream"
              />
          </div>
          <label for="package-file">
              {t("Package file")}
              <input id="package-file" type="file" name="file" on:change=select />
          </label>
          <Show when=move || !encoded.get().is_empty()>
              <small>{t("Selected file content will be uploaded as base64.")}</small>
          </Show>
          <label for="package-content">
              {t("Inline file content fallback")}
              <textarea
                  id="package-content"
                  rows="5"
                  prop:value=move || inline.get()
                  on:input=move |e| inline.set(event_target_value(&e))
                  placeholder="Artifact content for local smoke testing"
              />
          </label>
          <Show when=move || !mutation.error.get().is_empty()>
              <p role="alert" class="error">
                  {move || mutation.error.get()}
              </p>
          </Show>
          <div class="packages-submit">
              <button
                  type="submit"
                  class="primary"
                  disabled=move || mutation.busy.get() || reading.get() || !can_write.get()
              >
                  {move || t(
                      if mutation.busy.get() {
                          "Uploading package file..."
                      } else {
                          "Upload file"
                      },
                  )}
              </button>
          </div>
      </form>
  }
}
#[component]
fn Input(
  id: &'static str,
  label: &'static str,
  value: RwSignal<String>,
  placeholder: &'static str,
) -> impl IntoView {
  view! {
      <label for=id>
          {t(label)}
          <input
              id=id
              placeholder=placeholder
              prop:value=move || value.get()
              on:input=move |e| value.set(event_target_value(&e))
          />
      </label>
  }
}
