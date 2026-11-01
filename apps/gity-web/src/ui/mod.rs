use crate::api;
use leptos::prelude::*;
use serde_json::Value;
mod actions;
mod fields;
mod form;
mod mutation;
mod permissions;
mod time;
pub use actions::Action;
pub use fields::{Field, field};
pub use form::EditorForm;
pub use mutation::Mutation;
pub use permissions::{allowed, role_label};
pub use time::{format_timestamp, relative_time};
#[derive(Clone, Copy)]
pub struct Query {
  pub data: RwSignal<Value>,
  pub error: RwSignal<String>,
  pub loading: RwSignal<bool>,
  pub refresh: RwSignal<u64>,
}
pub fn query(path: String) -> Query {
  query_if(path, true)
}
pub fn query_if(path: String, enabled: bool) -> Query {
  query_fn(move || path.clone(), enabled)
}
pub fn query_fn(path: impl Fn() -> String + Send + Sync + 'static, enabled: bool) -> Query {
  query_with(path, enabled, false)
}
pub fn query_all_pages(path: String) -> Query {
  query_with(move || path.clone(), true, true)
}
fn query_with(
  path: impl Fn() -> String + Send + Sync + 'static,
  enabled: bool,
  all_pages: bool,
) -> Query {
  let q = Query {
    data: RwSignal::new(Value::Null),
    error: RwSignal::new(String::new()),
    loading: RwSignal::new(true),
    refresh: RwSignal::new(0),
  };
  if !enabled {
    q.loading.set(false);
    return q;
  }
  let sequence = RwSignal::new(0u64);
  Effect::new(move |_| {
    q.refresh.get();
    sequence.update(|v| *v += 1);
    let request_id = sequence.get_untracked();
    q.loading.set(true);
    q.error.set(String::new());
    let path = path();
    if path.is_empty() {
      q.loading.set(false);
      return;
    }
    wasm_bindgen_futures::spawn_local(async move {
      let result = if all_pages {
        api::list_all_pages(&path, move || {
          q.data.is_disposed() || sequence.get_untracked() != request_id
        })
        .await
      } else {
        api::request("GET", &path, None).await
      };
      if q.data.is_disposed() || sequence.get_untracked() != request_id {
        return;
      }
      match result {
        Ok(data) => q.data.set(data),
        Err(e) => q.error.set(e),
      };
      q.loading.set(false);
    });
  });
  q
}
#[component]
pub fn Status(q: Query) -> impl IntoView {
  view! {
      <Show when=move || q.loading.get()>
          <thaw::Spinner label="Loading…" size=thaw::SpinnerSize::Small />
      </Show>
      <QueryError q=q />
  }
}
#[component]
pub fn QueryError(q: Query) -> impl IntoView {
  view! {
      <Show when=move || !q.error.get().is_empty()>
          <p role="alert" class="error">
              {move || q.error.get()}
          </p>
      </Show>
  }
}
pub fn normalized(value: &Value) -> Value {
  for key in [
    "project_job",
    "release",
    "page",
    "package",
    "runner",
    "merge_request",
    "comment",
  ] {
    if value.get(key).is_some_and(Value::is_object) {
      let mut base = value[key].clone();
      if let Some(obj) = base.as_object_mut()
        && let Some(outer) = value.as_object()
      {
        for (k, v) in outer {
          if k != key {
            obj.insert(k.clone(), v.clone());
          }
        }
      }
      return base;
    }
  }
  value.clone()
}
