use super::*;
use serde_json::Value;
#[component]
pub(super) fn Output(job: Value, status: Status) -> impl IntoView {
  let result = api::text(&job, "result");
  view! {
      <section class="job-output">
          <header>
              <strong>
                  <thaw::Icon icon=icondata_lu::LuScrollText width="16px" height="16px" />
                  {t("Trace and artifacts")}
              </strong>
              <div class="jobs-badges">
                  <span class="badge outline">{t("Job ID")}": "{api::text(&job, "id")}</span>
                  <span class="badge outline">
                      {t("Attempts")}": "
                      {format!(
                          "{}/{}",
                          job["attempts"].as_u64().unwrap_or_default(),
                          job["max_attempts"].as_u64().filter(|value| *value > 0).unwrap_or(1),
                      )}
                  </span>
              </div>
          </header>
          <p>{t(status.guidance())}</p>
          {if result.is_empty() {
              view! {
                  <aside class="jobs-notice">
                      <strong>{t("No stored job output")}</strong>
                      <p>{t(status.empty_output())}</p>
                  </aside>
              }
                  .into_any()
          } else {
              view! { <pre>{result}</pre> }.into_any()
          }}
          <p class="job-artifact-hint">
              <thaw::Icon icon=icondata_lu::LuFileArchive width="16px" height="16px" />
              {t(
                  "Artifacts are produced by CI script jobs. If none appear, confirm the pipeline job declares artifact paths and the script reaches upload.",
              )}
          </p>
      </section>
  }
}
