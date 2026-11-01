use super::fields::FormField;
use super::*;
use serde_json::json;
use wasm_bindgen::JsCast;
#[component]
pub fn EditorForm(
  title: String,
  path: String,
  #[prop(default="POST".into())] method: String,
  fields: Vec<Field>,
  #[prop(default=Value::Null)] initial: Value,
  refresh: RwSignal<u64>,
  #[prop(default = false)] embedded: bool,
  #[prop(optional)] on_saved: Option<Callback<Value>>,
) -> impl IntoView {
  let busy = RwSignal::new(false);
  let error = RwSignal::new(String::new());
  let success = RwSignal::new(String::new());
  let specs = fields.clone();
  let submit = move |ev: leptos::ev::SubmitEvent| {
    ev.prevent_default();
    if busy.get_untracked() {
      return;
    }
    let form: web_sys::HtmlFormElement = ev.target().unwrap().unchecked_into();
    let Ok(data) = web_sys::FormData::new_with_form(&form) else {
      return;
    };
    let mut body = serde_json::Map::new();
    for f in &specs {
      let raw = data.get(f.key).as_string().unwrap_or_default();
      let value = match f.kind {
        "checkbox" => json!(data.has(f.key)),
        "number" => match raw.parse::<u64>() {
          Ok(n) => json!(n),
          Err(_) => {
            error.set(format!("{} must be a nonnegative number", f.label));
            return;
          }
        },
        "array" => json!(
          raw
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
        ),
        "json" => match serde_json::from_str::<Value>(&raw) {
          Ok(v) => v,
          Err(e) => {
            error.set(format!("{}: {e}", f.label));
            return;
          }
        },
        _ => json!(raw),
      };
      if f.required && value.as_str().is_some_and(|s| s.trim().is_empty()) {
        error.set(format!("{} is required", f.label));
        return;
      }
      if !f.required && value.as_str() == Some("") {
        continue;
      }
      body.insert(f.key.into(), value);
    }
    busy.set(true);
    error.set(String::new());
    success.set(String::new());
    let path = path.clone();
    let method = method.clone();
    wasm_bindgen_futures::spawn_local(async move {
      let result = api::request(&method, &path, Some(Value::Object(body))).await;
      if busy.is_disposed() {
        return;
      }
      busy.set(false);
      match result {
        Ok(result) => {
          success.set(crate::i18n::t("Saved successfully."));
          refresh.update(|n| *n += 1);
          if let Some(callback) = on_saved {
            callback.run(result);
          }
        }
        Err(e) => error.set(e),
      };
    });
  };
  view! {
      <details class="composer" class:embedded=embedded open=embedded>
          <summary>{crate::i18n::t(&title)}</summary>
          <form on:submit=submit>
              <div class="form-grid">
                  {fields
                      .into_iter()
                      .map(|f| {
                          view! { <FormField f=f initial=initial.clone() title=title.clone() /> }
                      })
                      .collect_view()}
              </div>
              <Show when=move || !error.get().is_empty()>
                  <p class="error" role="alert">
                      {move || error.get()}
                  </p>
              </Show>
              <Show when=move || !success.get().is_empty()>
                  <p class="success" role="status">
                      {move || success.get()}
                  </p>
              </Show>
              <thaw::Button
                  appearance=thaw::ButtonAppearance::Primary
                  button_type=thaw::ButtonType::Submit
                  loading=Signal::derive(move || busy.get())
                  disabled=Signal::derive(move || busy.get())
              >
                  {crate::i18n::t("Save")}
              </thaw::Button>
          </form>
      </details>
  }
}
