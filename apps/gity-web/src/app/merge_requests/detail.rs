use super::*;
use serde_json::json;

#[derive(Clone, Copy)]
pub(super) struct Review {
  pub context: Context,
  pub path: StoredValue<String>,
  pub mr: StoredValue<Value>,
  pub checks: Query,
  pub diff: Query,
  pub participants: Query,
  pub comments: Query,
  pub approvals: Query,
  pub identity: Query,
}
impl Review {
  pub fn people(self, role: &str) -> Vec<Value> {
    model::list(&self.participants.data.get(), "participants")
      .into_iter()
      .filter(|row| api::text(row, "role") == role)
      .collect()
  }
  pub fn approved(self) -> bool {
    let id = api::text(&self.identity.data.get(), "id");
    !id.is_empty()
      && model::list(&self.approvals.data.get(), "approvals")
        .iter()
        .any(|row| api::text(row, "user_id") == id)
  }
}

#[component]
pub(super) fn Detail(context: Context, mr: Value) -> impl IntoView {
  let path = format!(
    "{}/merge-requests/{}",
    context.path.get_value(),
    api::text(&mr, "iid")
  );
  let review = Review {
    context,
    path: StoredValue::new(path.clone()),
    mr: StoredValue::new(mr.clone()),
    checks: query(format!("{path}/checks")),
    diff: query(format!("{path}/diff")),
    participants: query(format!("{path}/participants")),
    comments: query(format!("{path}/comments")),
    approvals: query(format!("{path}/approvals")),
    identity: query("/users/me".into()),
  };
  let mutation = Mutation::new(context.rows.refresh);
  let state = api::text(&mr, "state");
  let opened = state == "opened";
  let closed = state == "closed";
  Effect::new(move |_| {
    context.rules_changed.get();
    review.checks.refresh.update(|n| *n += 1);
  });
  Effect::new(move |_| {
    context.gates.set(review.checks.data.get());
    context.gates_loading.set(review.checks.loading.get());
    context.gates_error.set(review.checks.error.get());
  });
  view! {
      <div class="mr-stack">
          <div class="mr-between mr-detail-heading">
              <div>
                  <div class="mr-tags">
                      <h3>{format!("!{} {}", api::text(&mr, "iid"), api::text(&mr, "title"))}</h3>
                      <Badge text=t(model::state_label(&state)) primary=opened />
                  </div>
                  <p class="mr-muted">
                      {format!(
                          "{} → {}",
                          api::text(&mr, "source_branch"),
                          api::text(&mr, "target_branch"),
                      )}
                  </p>
              </div>
              <div class="mr-tags">
                  <button
                      type="button"
                      class="mr-ghost"
                      on:click=move |_| review.diff.refresh.update(|v| *v += 1)
                  >
                      {t("Reload diff")}
                  </button>
                  <Show when=move || opened>
                      <button
                          type="button"
                          disabled=move || mutation.busy.get() || !context.can("write", 30)
                          on:click=move |_| {
                              mutation
                                  .run(
                                      "PATCH",
                                      review.path.get_value(),
                                      json!({"state":"closed"}),
                                      None,
                                  )
                          }
                      >
                          {t("Close merge request")}
                      </button>
                      <button
                          type="button"
                          class="primary"
                          disabled=move || {
                              mutation.busy.get() || !context.can("merge", 40)
                                  || review.checks.loading.get()
                                  || !review.checks.error.get().is_empty()
                                  || review.checks.data.get()["mergeable"] != true
                          }
                          on:click=move |_| {
                              mutation
                                  .run(
                                      "POST",
                                      format!("{}/merge", review.path.get_value()),
                                      json!({}),
                                      None,
                                  )
                          }
                      >
                          <thaw::Icon icon=icondata_lu::LuGitMerge width="14px" height="14px" />
                          {move || t(if mutation.busy.get() { "Merging..." } else { "Merge" })}
                      </button>
                  </Show>
                  <Show when=move || closed>
                      <button
                          type="button"
                          disabled=move || mutation.busy.get() || !context.can("write", 30)
                          on:click=move |_| {
                              mutation
                                  .run(
                                      "PATCH",
                                      review.path.get_value(),
                                      json!({"state":"opened"}),
                                      None,
                                  )
                          }
                      >
                          {t("Reopen merge request")}
                      </button>
                  </Show>
              </div>
          </div>
          <MutationError mutation=mutation />
          {if api::text(&mr, "description").is_empty() {
              None
          } else {
              Some(
                  view! {
                      <div class="mr-description mr-box">
                          <crate::markdown::Markdown content=api::text(&mr, "description") />
                      </div>
                  },
              )
          }}
          <Readiness review=review />
          <div class="mr-review-grid">
              <div class="mr-stack">
                  <checks::Checks review=review />
                  <comments::Comments review=review />
                  <div class="mr-stack mr-diff">
                      <strong>{t("Diff")}</strong>
                      <Status q=review.diff />
                      {move || {
                          let diff = api::text(&review.diff.data.get(), "diff");
                          if diff.trim().is_empty() {
                              view! { <pre>{t("No diff available.")}</pre> }.into_any()
                          } else {
                              view! { <pre>{diff}</pre> }.into_any()
                          }
                      }}
                  </div>
              </div>
              <div class="mr-stack">
                  <collaboration::Participants review=review />
                  <collaboration::Approvals review=review />
              </div>
          </div>
      </div>
  }
}

#[component]
fn Readiness(review: Review) -> impl IntoView {
  view! {
      <div class="mr-box mr-stack">
          <div class="mr-between">
              <div>
                  <strong>{t("Merge readiness")}</strong>
                  <p class="mr-muted mr-small">
                      {t("GitLab-style merge gate summary for reviewers and maintainers.")}
                  </p>
              </div>
              {move || {
                  view! {
                      <Badge
                          text=t(
                              model::readiness(&review.mr.get_value(), &review.checks.data.get()),
                          )
                          primary=review.checks.data.get()["mergeable"] == true
                      />
                  }
              }}
          </div>
          <div class="mr-readiness-stats">
              <Summary
                  label="Approvals"
                  value=Signal::derive(move || {
                      format!(
                          "{}/{}",
                          api::text(&review.checks.data.get(), "approval_count"),
                          api::text(&review.checks.data.get(), "required_approvals"),
                      )
                  })
                  detail=Signal::derive(move || t(
                      if review.approved() { "You approved" } else { "Your approval pending" },
                  ))
              />
              <Summary
                  label="Reviewers"
                  value=Signal::derive(move || review.people("reviewer").len().to_string())
                  detail=Signal::derive(move || {
                      if review.people("assignee").is_empty() {
                          t("No assignee")
                      } else {
                          format!("{} {}", review.people("assignee").len(), t("assignee(s)"))
                      }
                  })
              />
              <Summary
                  label="Pipeline"
                  value=Signal::derive(move || {
                      let checks = review.checks.data.get();
                      let status = api::text(&checks["pipeline"], "status");
                      if status.is_empty() { api::text(&checks, "status") } else { status }
                  })
                  detail=Signal::derive(move || t(
                      if review.checks.data.get()["pipeline_required"] == true {
                          "Required"
                      } else {
                          "Optional"
                      },
                  ))
              />
              <Summary
                  label="Discussion"
                  value=Signal::derive(move || {
                      model::list(&review.comments.data.get(), "comments").len().to_string()
                  })
                  detail=Signal::stored(t("comment(s)"))
              />
          </div>
      </div>
  }
}
#[component]
fn Summary(label: &'static str, value: Signal<String>, detail: Signal<String>) -> impl IntoView {
  view! {
      <div class="mr-summary">
          <small>{t(label)}</small>
          <strong>{move || value.get()}</strong>
          <small>{move || detail.get()}</small>
      </div>
  }
}
