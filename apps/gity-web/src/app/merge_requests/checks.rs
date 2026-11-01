use super::{detail::Review, *};

#[component]
pub(super) fn Checks(review: Review) -> impl IntoView {
  view! {
      <div class="mr-box mr-stack">
          <div class="mr-between">
              <div>
                  <div class="mr-tags">
                      <thaw::Icon icon=icondata_lu::LuShieldCheck width="16px" height="16px" />
                      <strong>{t("Merge checks")}</strong>
                      {move || {
                          let checks = review.checks.data.get();
                          let label = if checks.is_null() {
                              "Not loaded"
                          } else if checks["required"] != true {
                              "No checks required"
                          } else if checks["mergeable"] == true {
                              "Ready to merge"
                          } else if checks["status"] == "missing" {
                              "Pipeline missing"
                          } else {
                              "Blocked"
                          };
                          view! { <Badge text=t(label) primary=checks["mergeable"] == true /> }
                      }}
                  </div>
                  <p class="mr-muted mr-small">
                      {move || {
                          let reason = api::text(&review.checks.data.get(), "blocking_reason");
                          if reason.is_empty() {
                              t("Branch policy and pipeline state for this merge request.")
                          } else {
                              reason
                          }
                      }}
                  </p>
              </div>
              <button
                  type="button"
                  class="mr-ghost"
                  on:click=move |_| review.checks.refresh.update(|v| *v += 1)
              >
                  {t("Reload checks")}
              </button>
          </div>
          <Status q=review.checks />
          <Show when=move || !review.checks.data.get().is_null()>
              <div class="mr-check-lines">
                  <CheckLine review=review field="target_branch" label="Target branch" />
                  <CheckLine review=review field="source_commit_sha" label="Source commit" />
                  <CheckLine
                      review=review
                      field="target_branch_protected"
                      label="Protected target"
                      boolean=true
                  />
                  <CheckLine
                      review=review
                      field="pipeline_required"
                      label="Pipeline required"
                      boolean=true
                  />
                  <CheckLine
                      review=review
                      field="require_merge_request"
                      label="Merge request required"
                      boolean=true
                  />
                  <CheckLine review=review field="status" label="Pipeline status" />
              </div>
          </Show>
          <Show when=move || !model::list(&review.checks.data.get(), "blockers").is_empty()>
              <div class="mr-box mr-stack">
                  <strong class="mr-small mr-muted">{t("Blocking reasons")}</strong>
                  {move || {
                      model::list(&review.checks.data.get(), "blockers")
                          .into_iter()
                          .map(|blocker| view! { <Blocker blocker=blocker /> })
                          .collect_view()
                  }}
              </div>
          </Show>
          <Show when=move || !model::list(&review.checks.data.get(), "approval_rules").is_empty()>
              <div class="mr-stack">
                  <strong class="mr-small mr-muted">
                      {move || {
                          format!(
                              "{}: {}/{}",
                              t("Approval rules"),
                              api::text(&review.checks.data.get(), "approval_count"),
                              api::text(&review.checks.data.get(), "required_approvals"),
                          )
                      }}
                  </strong>
                  <div class="mr-check-lines">
                      {move || {
                          model::list(&review.checks.data.get(), "approval_rules")
                              .into_iter()
                              .map(|rule| {
                                  view! {
                                      <div class="mr-summary">
                                          <div class="mr-between">
                                              <strong>{api::text(&rule, "name")}</strong>
                                              <Badge
                                                  text=format!(
                                                      "{}/{}",
                                                      api::text(&rule, "approval_count"),
                                                      api::text(&rule, "approvals_required"),
                                                  )
                                                  primary=rule["satisfied"] == true
                                              />
                                          </div>
                                          <small>
                                              {format!(
                                                  "{} · {}",
                                                  t(
                                                      if rule["code_owner"] == true {
                                                          "CODEOWNERS"
                                                      } else {
                                                          "Eligible users"
                                                      },
                                                  ),
                                                  api::text(&rule, "target_branch"),
                                              )}
                                          </small>
                                      </div>
                                  }
                              })
                              .collect_view()
                      }}
                  </div>
              </div>
          </Show>
      </div>
  }
}
#[component]
fn CheckLine(
  review: Review,
  field: &'static str,
  label: &'static str,
  #[prop(default = false)] boolean: bool,
) -> impl IntoView {
  view! {
      <div class="mr-check-line">
          <strong>{t(label)}": "</strong>
          {move || {
              let checks = review.checks.data.get();
              if boolean {
                  t(if checks[field] == true { "Yes" } else { "No" })
              } else {
                  let value = api::text(&checks, field);
                  if field == "source_commit_sha" {
                      value.chars().take(8).collect()
                  } else {
                      t(&value)
                  }
              }
          }}
      </div>
  }
}
#[component]
fn Blocker(blocker: Value) -> impl IntoView {
  let code = api::text(&blocker, "code");
  let (title, hints, action) = match code.as_str() {
    "approval_rule_unsatisfied" => (
      "Approval requirements are not met",
      vec![
        "Have a required reviewer approve this merge request.",
        "Or adjust approval rules for the target branch.",
      ],
      Some((
        "#merge-request-approval-rules-panel",
        "Review approval rules",
      )),
    ),
    "pipeline_missing" => (
      "Pipeline not found",
      vec![
        "Push a new commit or fix CI config to trigger a new pipeline.",
        "Check the pipeline tab for queued or recent pipeline runs.",
      ],
      Some(("?tab=pipelines", "Open pipelines")),
    ),
    "pipeline_not_succeeded" => (
      "Pipeline has not succeeded",
      vec![
        "Fix pipeline failures and rerun the failed pipeline.",
        "Open pipeline logs from the Pipelines tab.",
      ],
      Some(("?tab=pipelines", "Open pipelines")),
    ),
    "pipeline_repository_missing" => (
      "Pipeline service is not configured",
      vec![
        "Enable/attach CI runtime for this repository.",
        "Create a CI config file in the source branch to trigger pipeline generation.",
      ],
      Some(("?tab=code", "Open code to add CI config")),
    ),
    _ => ("Policy check blocked", vec![], None),
  };
  view! {
      <div class="mr-blocker">
          <div class="mr-tags">
              <Badge text=t(&api::text(&blocker, "category")) />
              <strong>{t(title)}</strong>
          </div>
          <ul>
              <li>{api::text(&blocker, "message")}</li>
              {hints.into_iter().map(|hint| view! { <li>{t(hint)}</li> }).collect_view()}
          </ul>
          {action.map(|(href, label)| view! { <a href=href>{t(label)}</a> })}
      </div>
  }
}
