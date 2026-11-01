use super::FileTarget;
use crate::{api, ui::*};
use leptos::prelude::*;
use serde_json::Value;
#[component]
pub(super) fn TreeNode(
  entry: Value,
  project_path: String,
  branch: RwSignal<String>,
  selected: Signal<String>,
  on_open: Callback<FileTarget>,
  depth: u32,
) -> AnyView {
  let expanded = RwSignal::new(false);
  let path = api::text(&entry, "path");
  let directory = matches!(
    api::text(&entry, "kind").as_str(),
    "tree" | "directory" | "dir"
  );
  let child_path = path.clone();
  let endpoint = project_path.clone();
  let recursive_path = StoredValue::new(project_path);
  let children = query_fn(
    move || {
      if directory && expanded.get() {
        format!(
          "{endpoint}/repository/tree?branch_name={}&path={}",
          api::enc(&branch.get()),
          api::enc(&child_path)
        )
      } else {
        String::new()
      }
    },
    true,
  );
  let button_path = path.clone();
  let active = path.clone();
  view! {
      <div class="tree-node">
          <button
              class:active=move || selected.get() == active
              style:padding-left=format!("{}px", depth * 14 + 12)
              on:click=move |_| {
                  if directory {
                      expanded.set(!expanded.get_untracked());
                  } else {
                      on_open
                          .run(FileTarget {
                              path: button_path.clone(),
                              line: 1,
                              column: 1,
                              match_length: 0,
                          });
                  }
              }
          >
              <span>
                  {move || {
                      if directory {
                          if expanded.get() { "⌄ ▱" } else { "› ▱" }
                      } else {
                          "  ·"
                      }
                  }}
              </span>
              {api::text(&entry, "name")}
          </button>
          <Show when=move || expanded.get()>
              <Status q=children />
              {move || {
                  api::list(&children.data.get())
                      .into_iter()
                      .map(|item| {
                          view! {
                              <TreeNode
                                  entry=item
                                  project_path=recursive_path.get_value()
                                  branch=branch
                                  selected=selected
                                  on_open=on_open
                                  depth=depth + 1
                              />
                          }
                      })
                      .collect_view()
              }}
          </Show>
      </div>
  }
  .into_any()
}
