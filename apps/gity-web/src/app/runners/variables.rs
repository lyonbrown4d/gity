use super::*;

#[component]
pub(super) fn Variables(context: Context) -> impl IntoView {
  let query = query(format!("{}/ci/variables", context.path.get_value()));
  let saving = RwSignal::new(false);
  let deleting = RwSignal::new(false);
  let busy = Signal::derive(move || saving.get() || deleting.get());
  let initial_loading = Signal::derive(move || query.loading.get() && query.data.get().is_null());
  view! {
      <section class="runner-box runner-variables">
          <div class="runner-between">
              <div class="runner-variable-heading">
                  <strong class="runner-actions">
                      <thaw::Icon icon=icondata_lu::LuKeyRound width="16px" height="16px" />
                      {t("CI variables")}
                  </strong>
                  <small>
                      {t(
                          "Masked variables are injected into script jobs and redacted from logs.",
                      )}
                  </small>
                  <div class="runner-actions">
                      <Count query=query label="Variables" field="" />
                      <Count query=query label="Masked" field="masked" />
                      <Count query=query label="Protected" field="protected" />
                  </div>
              </div>
              <Reload query=query />
          </div>
          <guide::Variables />
          <Show when=move || !context.permissions.loading.get() && !context.can_admin()>
              <aside class="runner-notice">
                  {t(
                      "Your current project role can inspect runners, but cannot change CI variables.",
                  )}
              </aside>
          </Show>
          <variable_form::VariableForm
              context=context
              query=query
              saving=saving
              deleting=deleting
          />
          <div class="runner-list">
              <Show when=move || initial_loading.get()>
                  <div
                      class="runner-variable-skeleton"
                      role="status"
                      aria-label=t("Loading...")
                  ></div>
                  <div class="runner-variable-skeleton"></div>
              </Show>
              <Show when=move || !query.error.get().is_empty()>
                  <Status q=query />
              </Show>
              <Show when=move || {
                  !initial_loading.get() && query.error.get().is_empty()
                      && api::list(&query.data.get()).is_empty()
              }>
                  <Empty context=context />
              </Show>
              {move || {
                  api::list(&query.data.get())
                      .into_iter()
                      .map(|variable| {
                          view! {
                              <VariableRow
                                  context=context
                                  query=query
                                  variable=variable
                                  busy=busy
                                  deleting=deleting
                              />
                          }
                      })
                      .collect_view()
              }}
          </div>
      </section>
  }
}
#[component]
fn Count(query: Query, label: &'static str, field: &'static str) -> impl IntoView {
  view! {
      <span class="runner-badge" class:secondary=field.is_empty()>
          {move || {
              let count = api::list(&query.data.get())
                  .iter()
                  .filter(|row| field.is_empty() || row[field] == true)
                  .count();
              format!("{}: {count}", t(label))
          }}
      </span>
  }
}
#[component]
fn Empty(context: Context) -> impl IntoView {
  view! {
      <aside class="runner-notice runner-empty">
          <strong>{t("No CI variables configured")}</strong>
          <p>
              {t(
                  "Add masked variables for tokens, deployment keys, and environment values that jobs need at runtime. Use protected variables for release credentials and re-run the pipeline after saving.",
              )}
          </p>
          <div class="runner-actions">
              <span class="runner-badge" class:secondary=move || context.can_admin()>
                  {move || t(
                      if context.can_admin() {
                          "Use the form above"
                      } else {
                          "Runner admin required"
                      },
                  )}
              </span>
              <span class="runner-badge">{t("Example")}": DEPLOY_TOKEN"</span>
          </div>
      </aside>
  }
}
#[component]
fn VariableRow(
  context: Context,
  query: Query,
  variable: Value,
  busy: Signal<bool>,
  deleting: RwSignal<bool>,
) -> impl IntoView {
  let key = api::text(&variable, "key");
  let masked = variable["masked"] == true;
  let protected = variable["protected"] == true;
  let updated = api::text(&variable, "updated_at");
  view! {
      <article class="runner-variable-row runner-box runner-between" class:protected=protected>
          <div class="runner-entry-body">
              <strong class="runner-variable-key">{key.clone()}</strong>
              <small>
                  {if updated.is_empty() {
                      t("Value is hidden after save")
                  } else {
                      format!("{}: {updated}", t("Updated"))
                  }}
              </small>
              <small>{t(model::variable_exposure(masked, protected))}</small>
              <div class="runner-actions">
                  <span class="runner-badge" class:secondary=masked>
                      {t(if masked { "masked" } else { "visible" })}
                  </span>
                  <span class="runner-badge">
                      {t(if protected { "protected" } else { "unprotected" })}
                  </span>
              </div>
          </div>
          <Action
              label="Delete".into()
              path=format!("{}/ci/variables/{}", context.path.get_value(), api::enc(&key))
              method="DELETE".into()
              refresh=query.refresh
              confirm=true
              confirmation_title="Delete CI variable?".into()
              confirmation_description="Jobs started after deletion will no longer receive this variable."
                  .into()
              confirmation_label="Delete".into()
              busy_state=deleting
              disabled=Signal::derive(move || !context.can_admin() || busy.get())
          >
              <thaw::Icon icon=icondata_lu::LuTrash2 width="14px" height="14px" />
              {t("Delete")}
          </Action>
      </article>
  }
}
