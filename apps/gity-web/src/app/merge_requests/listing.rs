use super::*;
use std::collections::BTreeMap;

#[component]
pub(super) fn Listing(context: Context) -> impl IntoView {
  view! {
      <div class="mr-stack">
          <div class="mr-box mr-stack">
              <div class="mr-filter-grid">
                  <div class="mr-search">
                      <thaw::Icon icon=icondata_lu::LuSearch width="16px" height="16px" />
                      <input
                          aria-label=t("Search title, branch, or description")
                          placeholder=t("Search title, branch, or description")
                          prop:value=move || context.search.get()
                          on:input=move |e| context.search.set(event_target_value(&e))
                      />
                  </div>
                  <select
                      aria-label=t("Status")
                      prop:value=move || context.state.get()
                      on:change=move |e| context.state.set(event_target_value(&e))
                  >
                      <option value="opened">{t("Open merge requests")}</option>
                      <option value="merged">{t("Merged merge requests")}</option>
                      <option value="closed">{t("Closed merge requests")}</option>
                      <option value="all">{t("All merge requests")}</option>
                  </select>
                  <BranchFilter
                      context=context
                      field="source_branch"
                      label="Any source branch"
                      selected=context.source
                  />
                  <BranchFilter
                      context=context
                      field="target_branch"
                      label="Any target branch"
                      selected=context.target
                  />
              </div>
              <div class="mr-between mr-muted mr-small">
                  <span>
                      {move || {
                          format!(
                              "{} {} {} {} {}",
                              t("Showing"),
                              context.filtered().len(),
                              t("of"),
                              api::list(&context.rows.data.get()).len(),
                              t("merge requests"),
                          )
                      }}
                  </span>
                  <Show when=move || {
                      context.state.get() != "opened" || !context.search.get().is_empty()
                          || !context.source.get().is_empty() || !context.target.get().is_empty()
                  }>
                      <button
                          type="button"
                          class="mr-ghost"
                          on:click=move |_| context.reset_filters()
                      >
                          {t("Clear filters")}
                      </button>
                  </Show>
              </div>
          </div>
          <div class="mr-between">
              <button
                  type="button"
                  disabled=move || !context.can("create", 30)
                  on:click=move |_| context.composer.update(|v| *v = !*v)
              >
                  <thaw::Icon icon=icondata_lu::LuGitPullRequest width="16px" height="16px" />
                  {move || t(
                      if context.composer.get() {
                          "Hide new merge request form"
                      } else {
                          "New merge request"
                      },
                  )}
              </button>
              <Reload q=context.rows />
          </div>
          <Show when=move || !context.permissions.loading.get() && !context.can("create", 30)>
              <p class="mr-box">
                  {t(
                      "Your current project role can inspect merge requests, but cannot create them.",
                  )}
              </p>
          </Show>
          <Show when=move || context.composer.get()>
              <composer::Composer context=context />
          </Show>
          <div class="mr-list">
              <Status q=context.rows />
              <Show when=move || {
                  !context.rows.loading.get() && context.rows.error.get().is_empty()
                      && context.filtered().is_empty()
              }>
                  <div class="mr-empty">
                      <strong>
                          {move || t(
                              if api::list(&context.rows.data.get()).is_empty() {
                                  "No merge requests yet."
                              } else {
                                  "No merge requests match these filters."
                              },
                          )}
                      </strong>
                      <p>
                          {move || t(
                              if api::list(&context.rows.data.get()).is_empty() {
                                  "Create a merge request to start branch review and merge triage."
                              } else {
                                  "Try another state, source branch, target branch, or search term."
                              },
                          )}
                      </p>
                      <button
                          type="button"
                          on:click=move |_| {
                              if api::list(&context.rows.data.get_untracked()).is_empty() {
                                  context.composer.set(true);
                              } else {
                                  context.reset_filters();
                              }
                          }
                          disabled=move || {
                              api::list(&context.rows.data.get()).is_empty()
                                  && !context.can("create", 30)
                          }
                      >
                          {move || t(
                              if api::list(&context.rows.data.get()).is_empty() {
                                  "Create merge request"
                              } else {
                                  "Clear filters"
                              },
                          )}
                      </button>
                  </div>
              </Show>
              {move || {
                  context
                      .filtered()
                      .into_iter()
                      .map(|mr| view! { <ListItem context=context mr=mr /> })
                      .collect_view()
              }}
          </div>
      </div>
  }
}

#[component]
fn BranchFilter(
  context: Context,
  field: &'static str,
  label: &'static str,
  selected: RwSignal<String>,
) -> impl IntoView {
  view! {
      <select
          aria-label=t(label)
          prop:value=move || selected.get()
          on:change=move |e| selected.set(event_target_value(&e))
      >
          <option value="">{t(label)}</option>
          {move || {
              let mut counts = BTreeMap::<String, usize>::new();
              for row in api::list(&context.rows.data.get()) {
                  *counts.entry(api::text(&row, field)).or_default() += 1;
              }
              counts
                  .into_iter()
                  .map(|(branch, count)| {
                      view! {
                          <option value=branch.clone()>{format!("{branch} ({count})")}</option>
                      }
                  })
                  .collect_view()
          }}
      </select>
  }
}

#[component]
fn ListItem(context: Context, mr: Value) -> impl IntoView {
  let iid = api::text(&mr, "iid");
  let current = iid.clone();
  let select = iid.clone();
  let state = api::text(&mr, "state");
  let gate_iid = iid.clone();
  let gate_state = state.clone();
  let blocker_iid = iid.clone();
  view! {
      <button
          type="button"
          class="mr-list-item"
          class:selected=move || context.selected.get() == current
          on:click=move |_| context.select(select.clone())
      >
          <div class="mr-between">
              <strong>{format!("!{} {}", iid, api::text(&mr, "title"))}</strong>
              <Badge text=t(model::state_label(&state)) primary=state == "opened" />
          </div>
          <small class="mr-muted">
              {format!("{} {}", t("updated"), relative_time(&api::text(&mr, "updated_at")))}
          </small>
          <div class="mr-tags">
              <Badge text=format!("{}: {}", t("Source"), api::text(&mr, "source_branch")) />
              <Badge text=format!("{}: {}", t("Target"), api::text(&mr, "target_branch")) />
              {move || {
                  let selected = context.selected.get() == gate_iid;
                  let checks = context.gates.get();
                  let current = api::text(&checks["merge_request"], "iid") == gate_iid;
                  let label = if gate_state == "merged" {
                      "Merged"
                  } else if gate_state == "closed" {
                      "Closed"
                  } else if !selected {
                      "Select to inspect gates"
                  } else if context.gates_loading.get() {
                      "Loading gates"
                  } else if !current || !context.gates_error.get().is_empty() {
                      "Gate status unavailable"
                  } else if checks["mergeable"] == true {
                      "Ready to merge"
                  } else if checks["required"] == true {
                      "Blocked"
                  } else {
                      "No blockers"
                  };
                  view! {
                      <Badge
                          text=t(label)
                          primary=selected && current && checks["mergeable"] == true
                      />
                      {if selected && current {
                          let approvals = if checks["required_approvals"].as_u64().unwrap_or(0)
                              > 0
                          {
                              format!(
                                  "{} {}/{}",
                                  t("Approvals"),
                                  api::text(&checks, "approval_count"),
                                  api::text(&checks, "required_approvals"),
                              )
                          } else {
                              t("No approvals required")
                          };
                          Some(view! { <Badge text=approvals /> })
                      } else {
                          None
                      }}
                  }
              }}
          </div>
          {move || {
              let checks = context.gates.get();
              let reason = api::text(&checks, "blocking_reason");
              if context.selected.get() == blocker_iid
                  && api::text(&checks["merge_request"], "iid") == blocker_iid
                  && !reason.is_empty()
              {
                  Some(view! { <small class="mr-muted">{reason}</small> })
              } else {
                  None
              }
          }}
      </button>
  }
}
