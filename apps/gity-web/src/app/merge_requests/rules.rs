use super::*;

#[derive(Clone, Copy)]
pub(super) struct RulesState {
  pub query: Query,
  pub editing: RwSignal<Value>,
  pub form: RwSignal<u64>,
}
impl RulesState {
  pub fn reset(self) {
    self.editing.set(Value::Null);
    self.form.update(|n| *n += 1);
  }
}
#[component]
pub(super) fn Rules(context: Context) -> impl IntoView {
  let state = RulesState {
    query: query(format!(
      "{}/merge-request-approval-rules",
      context.path.get_value()
    )),
    editing: RwSignal::new(Value::Null),
    form: RwSignal::new(0),
  };
  Effect::new(move |_| {
    state.query.data.get();
    context.rules_changed.update(|n| *n += 1);
  });
  view! {
      <section class="mr-box mr-stack" id="merge-request-approval-rules-panel">
          <div class="mr-between">
              <div>
                  <div class="mr-tags">
                      <thaw::Icon icon=icondata_lu::LuShieldCheck width="16px" height="16px" />
                      <strong>{t("Approval rules")}</strong>
                  </div>
                  <p class="mr-small mr-muted">
                      {t("Configure branch scoped approvals and CODEOWNERS requirements.")}
                  </p>
              </div>
              <Reload q=state.query />
          </div>
          <div class="mr-rule-stats">
              <Stat
                  label="Rules"
                  value=Signal::derive(move || api::list(&state.query.data.get()).len())
              />
              <Stat
                  label="Branch scoped"
                  value=Signal::derive(move || {
                      api::list(&state.query.data.get())
                          .iter()
                          .filter(|row| {
                              !["", "*"].contains(&api::text(row, "target_branch").as_str())
                          })
                          .count()
                  })
              />
              <Stat
                  label="CODEOWNERS"
                  value=Signal::derive(move || {
                      api::list(&state.query.data.get())
                          .iter()
                          .filter(|row| row["code_owner"] == true)
                          .count()
                  })
              />
          </div>
          <Show when=move || !context.permissions.loading.get() && !context.can("merge", 40)>
              <p class="mr-box">
                  {t(
                      "Your current project role can inspect approval rules, but cannot change them.",
                  )}
              </p>
          </Show>
          {move || {
              state.form.get();
              view! { <rule_form::RuleForm context=context state=state /> }
          }}
          <Status q=state.query />
          <div class="mr-rule-table">
              <table>
                  <thead>
                      <tr>
                          <th>{t("Rule")}</th>
                          <th>{t("Scope")}</th>
                          <th>{t("Approvers")}</th>
                          <th>{t("Requirement")}</th>
                          <th>{t("Actions")}</th>
                      </tr>
                  </thead>
                  <tbody>
                      <Show when=move || {
                          !state.query.loading.get()
                              && api::list(&state.query.data.get()).is_empty()
                      }>
                          <tr>
                              <td colspan="5" class="mr-muted">
                                  {t("No approval rules configured.")}
                              </td>
                          </tr>
                      </Show>
                      {move || {
                          api::list(&state.query.data.get())
                              .into_iter()
                              .map(|rule| {
                                  view! { <RuleRow context=context state=state rule=rule /> }
                              })
                              .collect_view()
                      }}
                  </tbody>
              </table>
          </div>
      </section>
  }
}
#[component]
fn RuleRow(context: Context, state: RulesState, rule: Value) -> impl IntoView {
  let edit = rule.clone();
  let eligible = model::strings(&rule["eligible_user_ids"]);
  view! {
      <tr>
          <td>
              <strong>{api::text(&rule, "name")}</strong>
              {if rule["code_owner"] == true {
                  Some(view! { <Badge text=t("CODEOWNERS") primary=true /> })
              } else {
                  None
              }}
          </td>
          <td>
              <Badge text=api::text(&rule, "target_branch") />
          </td>
          <td class="mr-muted">
              {if eligible.is_empty() {
                  t("Any approver can satisfy this rule.")
              } else {
                  eligible
                      .iter()
                      .map(|id| model::user(&context.users.data.get(), id))
                      .collect::<Vec<_>>()
                      .join(", ")
              }}
          </td>
          <td>
              <Badge text=format!(
                  "{} {}",
                  api::text(&rule, "approvals_required"),
                  t("approval(s)"),
              ) />
          </td>
          <td>
              <div class="mr-tags mr-end">
                  <button
                      type="button"
                      disabled=move || !context.can("merge", 40)
                      on:click=move |_| {
                          state.editing.set(edit.clone());
                          state.form.update(|n| *n += 1);
                      }
                  >
                      {t("Edit")}
                  </button>
                  <Action
                      label="Delete".into()
                      path=format!(
                          "{}/merge-request-approval-rules/{}",
                          context.path.get_value(),
                          api::text(&rule, "id"),
                      )
                      method="DELETE".into()
                      refresh=state.query.refresh
                      confirm=true
                      disabled=Signal::derive(move || !context.can("merge", 40))
                      confirmation_title="Delete approval rule?".into()
                      confirmation_description="Existing merge request approvals remain, but future checks will ignore this rule."
                          .into()
                  />
              </div>
          </td>
      </tr>
  }
}
