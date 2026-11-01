use super::FileTarget;
use crate::api;
use leptos::prelude::*;
use serde_json::Value;
#[component]
pub(super) fn SourceSearch(
  project_path: String,
  branch: RwSignal<String>,
  on_open: Callback<FileTarget>,
) -> impl IntoView {
  let term = RwSignal::new(String::new());
  let filter = RwSignal::new(String::new());
  let match_case = RwSignal::new(false);
  let regex = RwSignal::new(false);
  let results = RwSignal::new(Vec::<Value>::new());
  let loading = RwSignal::new(false);
  let error = RwSignal::new(String::new());
  let search = move |ev: leptos::ev::SubmitEvent| {
    ev.prevent_default();
    if loading.get_untracked() {
      return;
    }
    let path = project_path.clone();
    let query = term.get_untracked();
    let p = filter.get_untracked();
    let b = branch.get_untracked();
    let sensitive = match_case.get_untracked();
    let pattern = regex.get_untracked();
    loading.set(true);
    error.set(String::new());
    wasm_bindgen_futures::spawn_local(async move {
      let result=api::request("GET",&format!("{path}/repository/search?branch_name={}&query={}&path={}&match_case={sensitive}&regex={pattern}",api::enc(&b),api::enc(&query),api::enc(&p)),None).await;
      if results.is_disposed() {
        return;
      }
      match result {
        Ok(v) => results.set(api::list(&v)),
        Err(e) => error.set(e),
      }
      loading.set(false);
    });
  };
  view! {
      <form class="source-search" on:submit=search>
          <input
              type="search"
              placeholder="Search in project"
              aria-label="Search text"
              on:input=move |e| term.set(event_target_value(&e))
          />
          <input
              placeholder="Path (optional)"
              aria-label="Path (optional)"
              on:input=move |e| filter.set(event_target_value(&e))
          />
          <div>
              <label>
                  <input
                      type="checkbox"
                      on:change=move |e| match_case.set(event_target_checked(&e))
                  />
                  {crate::i18n::t("Match case")}
              </label>
              <label>
                  <input type="checkbox" on:change=move |e| regex.set(event_target_checked(&e)) />
                  {crate::i18n::t("Regular expression")}
              </label>
              <button disabled=move || loading.get()>{crate::i18n::t("Search")}</button>
          </div>
      </form>
      <Show when=move || !error.get().is_empty()>
          <p role="alert" class="error">
              {move || error.get()}
          </p>
      </Show>
      <Show when=move || !results.get().is_empty()>
          <div class="source-results">
              <p>{move || results.get().len()}" result(s)"</p>
              {move || {
                  results
                      .get()
                      .into_iter()
                      .map(|r| {
                          let target = FileTarget {
                              path: api::text(&r, "path"),
                              line: r["line_number"].as_u64().unwrap_or(1) as u32,
                              column: r["column"].as_u64().unwrap_or(1) as u32,
                              match_length: r["match_length"].as_u64().unwrap_or(0) as u32,
                          };
                          view! {
                              <button on:click=move |_| on_open.run(target.clone())>
                                  <strong>
                                      {api::text(&r, "path")}":"{api::text(&r, "line_number")}
                                  </strong>
                                  <small>{api::text(&r, "line_content")}</small>
                              </button>
                          }
                      })
                      .collect_view()
              }}
          </div>
      </Show>
  }
}
