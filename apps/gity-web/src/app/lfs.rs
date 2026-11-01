mod locks;
mod objects;
mod setup;
use crate::{api, i18n::t, ui::*};
use leptos::prelude::*;
use serde_json::Value;

fn rows(query: Query, key: &str) -> Vec<Value> {
  query.data.get()[key]
    .as_array()
    .cloned()
    .unwrap_or_default()
}
fn bytes(value: u64) -> String {
  if value < 1024 {
    format!("{value} B")
  } else if value < 1024 * 1024 {
    format!("{:.1} KiB", value as f64 / 1024.0)
  } else {
    format!("{:.1} MiB", value as f64 / 1048576.0)
  }
}

#[component]
pub(super) fn Lfs(
  project_path: String,
  project: RwSignal<Value>,
  permissions: Query,
) -> impl IntoView {
  let objects = query(format!("{project_path}/lfs/objects"));
  let locks = query(format!("{project_path}/lfs/locks"));
  let can_write =
    Signal::derive(move || allowed(&permissions.data.get(), "project.write", "can_write", 30));
  view! {
      <section class="lfs-panel account-card card-enter">
          <header class="lfs-header">
              <div>
                  <h2>{t("Git LFS")}</h2>
                  <p>{t("Inspect stored LFS objects and manage project file locks.")}</p>
              </div>
              <span
                  class="badge"
                  class:outline=move || !can_write.get()
                  class:secondary=move || can_write.get()
              >
                  {move || t(if can_write.get() { "Locks enabled" } else { "Read only" })}
              </span>
          </header>
          <div class="lfs-body">
              <div class="lfs-stats">
                  <Stat
                      label="Objects"
                      value=Signal::derive(move || rows(objects, "objects").len().to_string())
                  />
                  <Stat
                      label="Storage"
                      value=Signal::derive(move || bytes(
                          rows(objects, "objects")
                              .iter()
                              .filter_map(|row| row["byte_size"].as_u64())
                              .sum(),
                      ))
                  />
                  <Stat
                      label="Locks"
                      value=Signal::derive(move || rows(locks, "locks").len().to_string())
                  />
              </div>
              <QueryError q=objects />
              <QueryError q=locks />
              <Show when=move || !permissions.loading.get() && !can_write.get()>
                  <aside class="lfs-notice">
                      <strong>{t("LFS locks are read-only")}</strong>
                      <p>
                          {t(
                              "Your current project role can inspect LFS, but cannot manage locks.",
                          )}
                      </p>
                  </aside>
              </Show>
              <div class="lfs-workspace">
                  <div class="lfs-main">
                      <div class="lfs-endpoint">
                          <div>
                              <strong>{t("LFS endpoint")}</strong>
                              <small>
                                  {move || {
                                      format!(
                                          "{}/info/lfs",
                                          api::text(&project.get(), "clone_http_url")
                                              .trim_end_matches('/'),
                                      )
                                  }}
                              </small>
                          </div>
                          <button
                              type="button"
                              on:click=move |_| {
                                  objects.refresh.update(|n| *n += 1);
                                  locks.refresh.update(|n| *n += 1);
                              }
                          >
                              <thaw::Icon
                                  icon=icondata_lu::LuRefreshCw
                                  width="16px"
                                  height="16px"
                              />
                              {t("Reload")}
                          </button>
                      </div>
                      <objects::Objects objects=objects />
                      <locks::Locks query=locks path=project_path can_write=can_write />
                  </div>
                  <setup::Setup />
              </div>
          </div>
      </section>
  }
}
#[component]
fn Stat(label: &'static str, value: Signal<String>) -> impl IntoView {
  view! {
      <div class="lfs-stat">
          <small>{t(label)}</small>
          <strong>{move || value.get()}</strong>
      </div>
  }
}
#[component]
fn Empty(title: &'static str, description: Signal<String>) -> impl IntoView {
  view! {
      <div class="lfs-empty">
          <thaw::Icon icon=icondata_lu::LuLock width="20px" height="20px" />
          <div>
              <strong>{t(title)}</strong>
              <p>{move || description.get()}</p>
          </div>
      </div>
  }
}
#[cfg(test)]
#[path = "../tests/lfs.rs"]
mod tests;
