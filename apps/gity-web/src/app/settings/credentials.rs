mod guide;
mod keys;
mod tokens;
use super::Header;
use crate::{i18n::t, ui::*};
use leptos::prelude::*;
use serde_json::Value;

#[derive(Clone, Copy)]
pub(crate) struct Context {
  pub access: Query,
  pub deploy: Query,
  pub keys: Query,
  pub mutation: Mutation,
  pub disabled: Signal<bool>,
  pub created: RwSignal<Value>,
  pub path: StoredValue<String>,
  pub project: Query,
}
#[component]
pub(super) fn Credentials(
  project_path: String,
  project: Query,
  permissions: Query,
) -> impl IntoView {
  let access = query(format!("{project_path}/access-tokens"));
  let deploy = query(format!("{project_path}/deploy-tokens"));
  let keys = query(format!("{project_path}/deploy-keys"));
  let refresh = RwSignal::new(0u64);
  let mutation = Mutation::new(refresh);
  Effect::new(move || {
    if refresh.get() > 0 {
      access.refresh.update(|v| *v += 1);
      deploy.refresh.update(|v| *v += 1);
      keys.refresh.update(|v| *v += 1);
    }
  });
  let can_admin = Signal::derive(move || {
    allowed(
      &permissions.data.get(),
      "project.repository.admin",
      "repository_admin",
      40,
    )
  });
  let disabled = Signal::derive(move || !can_admin.get() || mutation.busy.get());
  let context = Context {
    access,
    deploy,
    keys,
    mutation,
    disabled,
    created: RwSignal::new(Value::Null),
    path: StoredValue::new(project_path),
    project,
  };
  view! {
      <section class="settings-card account-card card-enter">
          <Header
              title="Project credentials"
              description="Manage project-scoped tokens and deploy keys for Git, packages and automation."
          />
          <div class="settings-body settings-credentials">
              <guide::AutomationGuide context=context />
              <Show when=move || !can_admin.get()>
                  <p class="settings-notice">
                      {t(
                          "Your current project role can inspect settings, but cannot change project credentials.",
                      )}
                  </p>
              </Show>
              <Show when=move || context.created.get().is_object()>
                  <guide::CreatedToken context=context />
              </Show>
              <Show when=move || !mutation.error.get().is_empty()>
                  <p class="error" role="alert">
                      {move || mutation.error.get()}
                  </p>
              </Show>
              <tokens::Tokens context=context deploy=false />
              <tokens::Tokens context=context deploy=true />
              <keys::Keys context=context />
          </div>
      </section>
  }
}
#[component]
fn SectionHeader(title: &'static str, description: &'static str, query: Query) -> impl IntoView {
  view! {
      <header class="settings-section-header">
          <div>
              <h3>{t(title)}</h3>
              <p>{t(description)}</p>
          </div>
          <button type="button" on:click=move |_| query.refresh.update(|v| *v += 1)>
              <thaw::Icon icon=icondata_lu::LuRefreshCw width="16px" height="16px" />
              {t("Reload")}
          </button>
      </header>
  }
}
