use super::*;

#[component]
pub(super) fn Item(label: String, description: String) -> impl IntoView {
  view! {
      <div class="runner-guide-item">
          <strong>{label}</strong>
          <p>{description}</p>
      </div>
  }
}
#[component]
pub(super) fn Trust() -> impl IntoView {
  view! {
      <aside class="runner-notice runner-guide">
          <strong>{t("Runner trust and matching")}</strong>
          <div class="runner-guide-grid">
              <Item
                  label=t("Trusted host")
                  description=t(
                      "Install runners only on machines allowed to read repository workspaces and injected variables.",
                  )
              />
              <Item
                  label=t("Runtime")
                  description=t(
                      "The runner host provides shell tools, containers, caches, and network access; keep that runtime patched and scoped.",
                  )
              />
              <Item
                  label=t("Tag matching")
                  description=t(
                      "Pending jobs need an active runner whose tags satisfy the job tags declared in the CI config.",
                  )
              />
          </div>
      </aside>
  }
}
#[component]
pub(super) fn Protocol() -> impl IntoView {
  view! {
      <aside class="runner-notice runner-guide runner-protocol">
          <strong>{t("Runner protocol")}</strong>
          <p>
              {t(
                  "Use the token with /runners/heartbeat, /runners/jobs/claim, /runners/jobs/{id}/complete, and /runners/jobs/{id}/fail.",
              )}
          </p>
          <p>
              {t(
                  "Treat runner tokens like deployment secrets. Runners execute repository scripts and can receive CI variables, so install them only on hosts you trust and can patch.",
              )}
          </p>
      </aside>
  }
}
#[component]
pub(super) fn Variables() -> impl IntoView {
  view! {
      <aside class="runner-notice runner-guide">
          <strong>{t("Masking and protection")}</strong>
          <div class="runner-guide-grid two">
              <Item
                  label=t("Masked")
                  description=t(
                      "Masked values are hidden after save and redacted from job logs, but jobs can still read them at runtime.",
                  )
              />
              <Item
                  label=t("Protected")
                  description=t(
                      "Protected marks secrets intended for protected refs or release jobs; keep deployment credentials protected and scoped.",
                  )
              />
          </div>
      </aside>
  }
}
