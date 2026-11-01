use crate::{api, ui::*};
use leptos::prelude::*;

#[component]
pub(super) fn AdminOverview() -> impl IntoView {
  let orgs = query("/orgs".into());
  let projects = query_fn(
    move || {
      if orgs.loading.get() {
        return String::new();
      }
      api::list(&orgs.data.get())
        .first()
        .map(|org| {
          format!(
            "/projects?organization_id={}",
            api::enc(&api::text(org, "id"))
          )
        })
        .unwrap_or_default()
    },
    true,
  );
  let error = move || {
    let org_error = orgs.error.get();
    if !org_error.is_empty() {
      org_error
    } else {
      projects.error.get()
    }
  };
  let loading = move || orgs.loading.get() || projects.loading.get();

  view! {
      <main class="admin-overview page-enter">
          <header class="admin-overview-header">
              <div>
                  <p class="admin-overview-eyebrow">{crate::i18n::t("Admin Side")}</p>
                  <h1>{crate::i18n::t("Admin Overview")}</h1>
                  <p>
                      {crate::i18n::t(
                          "Monitor organizations, projects, and current control-plane baseline.",
                      )}
                  </p>
              </div>
          </header>
          <Show when=move || !error().is_empty()>
              <div class="admin-overview-error" role="alert">
                  <span>{move || error()}</span>
              </div>
          </Show>
          <dl class="admin-overview-metrics">
              <Metric
                  label="Organizations"
                  value=move || {
                      if loading() {
                          "--".into()
                      } else {
                          api::list(&orgs.data.get()).len().to_string()
                      }
                  }
                  detail="total"
                  active=move || !api::list(&orgs.data.get()).is_empty()
              />
              <Metric
                  label="Projects"
                  value=move || {
                      if loading() {
                          "--".into()
                      } else {
                          api::list(&projects.data.get()).len().to_string()
                      }
                  }
                  detail="projects"
                  active=move || !api::list(&projects.data.get()).is_empty()
              />
              <Metric
                  label="Baseline"
                  value=|| "03".into()
                  detail="System Composition"
                  active=move || error().is_empty()
              />
          </dl>
          <div class="admin-overview-columns">
              <section class="admin-overview-panel admin-overview-snapshot">
                  <header>
                      <div>
                          <h2>{crate::i18n::t("Organization Snapshot")}</h2>
                          <p>{crate::i18n::t("Quick view of organizations and role mapping.")}</p>
                      </div>
                      <span class="admin-overview-meta">
                          {move || {
                              format!(
                                  "{} {}",
                                  api::list(&orgs.data.get()).len(),
                                  crate::i18n::t("total"),
                              )
                          }}
                      </span>
                  </header>
                  {move || {
                      if loading() {
                          view! {
                              <div class="admin-overview-loading" aria-label="Loading">
                                  <span></span>
                                  <span></span>
                                  <span></span>
                                  <span></span>
                              </div>
                          }
                              .into_any()
                      } else {
                          let rows = api::list(&orgs.data.get());
                          view! {
                              <>
                                  <table>
                                      <thead>
                                          <tr>
                                              <th>{crate::i18n::t("Organization")}</th>
                                              <th>{crate::i18n::t("Role")}</th>
                                              <th class="admin-overview-id">"ID"</th>
                                          </tr>
                                      </thead>
                                      <tbody>
                                          {rows
                                              .iter()
                                              .map(|org| {
                                                  view! {
                                                      <tr>
                                                          <td>
                                                              <div class="admin-overview-org">
                                                                  <i></i>
                                                                  <div>
                                                                      <p>{api::text(org, "name")}</p>
                                                                      <small>{api::text(org, "key")}</small>
                                                                  </div>
                                                              </div>
                                                          </td>
                                                          <td>
                                                              <span class="admin-overview-badge">
                                                                  {api::text(org, "role")}
                                                              </span>
                                                          </td>
                                                          <td class="admin-overview-id">{api::text(org, "id")}</td>
                                                      </tr>
                                                  }
                                              })
                                              .collect_view()}
                                      </tbody>
                                  </table>
                                  {if rows.is_empty() {
                                      view! {
                                          <div class="admin-overview-empty">
                                              {crate::i18n::t("No organizations available.")}
                                          </div>
                                      }
                                          .into_any()
                                  } else {
                                      ().into_any()
                                  }}
                              </>
                          }
                              .into_any()
                      }
                  }}
              </section>
              <section class="admin-overview-panel admin-overview-composition">
                  <header>
                      <div>
                          <h2>{crate::i18n::t("System Composition")}</h2>
                          <p>
                              {crate::i18n::t(
                                  "Current baseline aligns route auth with role-scoped layouts and CRUD pages.",
                              )}
                          </p>
                      </div>
                  </header>
                  <div class="admin-overview-stack">
                      <Composition name="Leptos" detail="Rust routing and authentication" />
                      <Composition name="Thaw" detail="Themed Rust components" />
                      <Composition name="split user/admin" detail="Access role" />
                  </div>
              </section>
          </div>
      </main>
  }
}

#[component]
fn Metric(
  label: &'static str,
  value: impl Fn() -> String + Send + Sync + 'static,
  detail: &'static str,
  active: impl Fn() -> bool + Send + Sync + 'static,
) -> impl IntoView {
  view! {
      <div>
          <dt>
              <i class:active=active></i>
              {crate::i18n::t(label)}
          </dt>
          <dd>
              <strong>{value}</strong>
              <span>{crate::i18n::t(detail)}</span>
          </dd>
      </div>
  }
}

#[component]
fn Composition(name: &'static str, detail: &'static str) -> impl IntoView {
  view! {
      <div>
          <i></i>
          <span>
              <strong>{crate::i18n::t(name)}</strong>
              <small>{crate::i18n::t(detail)}</small>
          </span>
          <b>{crate::i18n::t("active")}</b>
      </div>
  }
}
