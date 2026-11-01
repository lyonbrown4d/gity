mod controller;
mod dialogs;
pub(super) mod forms;
pub(super) mod model;
pub(super) mod panels;
pub(super) mod scope;
pub(super) mod selector;
use super::admin_ui::{ConsoleHeader, ConsoleMetric, ConsoleMetrics};
use crate::i18n::t;
use controller::OrganizationAdmin;
use leptos::prelude::*;

#[component]
pub(super) fn AdminOrganizations() -> impl IntoView {
  let state = OrganizationAdmin::new();
  view! {
      <div class="admin-organizations console-page page-enter">
          <ConsoleHeader
              title="Organizations"
              description="Full CRUD for organizations and member management entry."
          >
              <button
                  type="button"
                  class="primary"
                  on:click=move |_| {
                      state.clear_errors();
                      state.create_open.set(true);
                  }
              >
                  <thaw::Icon icon=icondata_lu::LuPlus width="14px" height="14px" />
                  {t("New Organization")}
              </button>
          </ConsoleHeader>
          <selector::OrganizationToolbar state=state />
          <Show when=move || !state.error.get().is_empty()>
              <p class="error admin-directory-error" role="alert">
                  {move || state.error.get()}
              </p>
          </Show>
          <ConsoleMetrics items=vec![
              ConsoleMetric {
                  label: "Organizations",
                  value: Signal::derive(move || state.rows.get().len().to_string()),
                  detail: t("total"),
                  active: Signal::derive(move || !state.rows.get().is_empty()),
              },
              ConsoleMetric {
                  label: "Members",
                  value: Signal::derive(move || state.member_rows.get().len().to_string()),
                  detail: t("member"),
                  active: Signal::derive(move || !state.member_rows.get().is_empty()),
              },
              ConsoleMetric {
                  label: "Projects",
                  value: Signal::derive(move || state.project_rows.get().len().to_string()),
                  detail: t("projects"),
                  active: Signal::derive(move || !state.project_rows.get().is_empty()),
              },
          ] />
          <panels::OrganizationPanels state=state />
          <dialogs::OrganizationDialogs state=state />
      </div>
  }
}
