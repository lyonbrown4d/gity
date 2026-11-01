use super::packages::Packages;
use crate::{i18n::t, ui::Query};
use leptos::prelude::*;

#[component]
pub(super) fn PackageAvailabilityGate(
  capabilities: Query,
  project_path: String,
  permissions: Query,
) -> impl IntoView {
  view! {
      {move || {
          let package_registry = capabilities.data.get()["package_registry"].as_bool();
          match package_registry {
              Some(true) => view! {
                  <Packages project_path=project_path.clone() permissions=permissions />
              }
                  .into_any(),
              Some(false) => view! {
                  <section class="packages-panel account-card card-enter">
                      <header class="packages-header">
                          <div>
                              <h2>{t("Package Registry")}</h2>
                              <p>{t("Package registry is unavailable in this deployment.")}</p>
                          </div>
                      </header>
                      <div class="packages-body">
                          <div class="packages-notice" role="status">
                              <strong>{t("Package features are disabled")}</strong>
                              <p>
                                  {t("This deployment does not provide package browsing or publishing.")}
                              </p>
                          </div>
                      </div>
                  </section>
              }
                  .into_any(),
              None if capabilities.loading.get() => view! {
                  <p role="status">{t("Checking package availability…")}</p>
              }
                  .into_any(),
              None => view! {
                  <section class="packages-panel account-card card-enter">
                      <header class="packages-header">
                          <div>
                              <h2>{t("Package Registry")}</h2>
                          </div>
                      </header>
                      <div class="packages-body">
                          <div class="packages-notice" role="alert">
                              <strong>{t("Package availability is unknown")}</strong>
                              <p>
                                  {t("The server did not confirm package support. Browsing and publishing are unavailable until it can be verified.")}
                              </p>
                          </div>
                      </div>
                  </section>
              }
                  .into_any(),
          }
      }}
  }
}
