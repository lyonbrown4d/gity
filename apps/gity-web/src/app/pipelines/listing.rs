use super::*;

#[component]
pub(super) fn Listing(context: Context) -> impl IntoView {
  view! {
      <div class="pipeline-stack pipeline-list-panel">
          <h3>{t("Pipelines")}</h3>
          <div class="pipeline-box pipeline-list">
              <QueryError q=context.rows />
              <Show when=move || context.rows.loading.get()>
                  <div class="pipeline-stack" style="gap:8px">
                      <div class="loading-placeholder" style="height:80px" />
                      <div class="loading-placeholder" style="height:80px" />
                      <div class="loading-placeholder" style="height:80px" />
                  </div>
              </Show>
              <Show when=move || {
                  !context.rows.loading.get() && context.rows.error.get().is_empty()
                      && context.pipelines.get().is_empty()
              }>
                  <aside class="pipeline-notice pipeline-empty pipeline-stack">
                      <strong>{t("No pipelines yet")}</strong>
                      <p>
                          {t(
                              "Create a manual pipeline to validate CI config, or push commits to let pipelines appear here automatically.",
                          )}
                      </p>
                      <div class="pipeline-actions">
                          <button
                              type="button"
                              class="primary"
                              disabled=move || !context.can_write()
                              on:click=move |_| context.composer.set(true)
                          >
                              <thaw::Icon icon=icondata_lu::LuPlay width="14px" height="14px" />
                              {t("Create first pipeline")}
                          </button>
                          <Show when=move || !context.can_write()>
                              <span class="pipeline-badge">
                                  {t("CI write permission required")}
                              </span>
                          </Show>
                      </div>
                  </aside>
              </Show>
              <For
                  each=move || context.pipelines.get()
                  key=|row| (row.id.clone(), row.updated.clone(), row.state)
                  children=move |row| {
                      let id = StoredValue::new(row.id.clone());
                      view! {
                          <button
                              type="button"
                              class="pipeline-list-row"
                              class:selected=move || context.selected.get() == id.get_value()
                              aria-pressed=move || context.selected.get() == id.get_value()
                              class:failed=row.state == State::Failed
                              on:click=move |_| context.select(id.get_value())
                          >
                              <div class="pipeline-between">
                                  <strong>{row.title()}</strong>
                                  <Badge state=row.state />
                              </div>
                              <p>
                                  <thaw::Icon
                                      icon=icondata_lu::LuGitBranch
                                      width="12px"
                                      height="12px"
                                  />
                                  {row.reference.clone()}
                                  " · "
                                  {row.short_commit()}
                                  " · "
                                  {relative_time(&row.updated)}
                              </p>

                          </button>
                      }
                  }
              />
          </div>
      </div>
  }
}
