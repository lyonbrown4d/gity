use super::super::account::AccountMenu;
use crate::ui::Query;
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub(super) fn PrimaryRail(me: Query, area: &'static str) -> impl IntoView {
  view! {
      <div class="primary-rail">
          <A href="/app/dashboard" {..} class="rail-brand" aria-label="Gity">
              <thaw::Icon icon=icondata_lu::LuArrowRightLeft width="16px" height="16px" />
          </A>
          <nav aria-label="Primary navigation">
              <A href="/app/dashboard" {..} title="Workspace" aria-label="Workspace" class:active=area
                  == "workspace">
                  <thaw::Icon icon=icondata_lu::LuHouse />
                  <small>{crate::i18n::t("Home")}</small>
              </A>
              <A href="/app/projects" {..} title="Projects" aria-label="Projects" class:active=area
                  == "projects">
                  <thaw::Icon icon=icondata_lu::LuFolderGit2 />
                  <small>{crate::i18n::t("Projects")}</small>
              </A>
              <A href="/app/todos" {..} title="Inbox" aria-label="Inbox" class:active=area == "inbox">
                  <thaw::Icon icon=icondata_lu::LuInbox />
                  <small>{crate::i18n::t("Inbox")}</small>
              </A>
          </nav>
          <div class="rail-footer">
              <Show when=move || me.data.get()["is_super_admin"] == true>
                  <A href="/admin" {..} title="Admin" aria-label="Admin">
                      <thaw::Icon icon=icondata_lu::LuShieldCheck />
                      <small>{crate::i18n::t("Admin")}</small>
                  </A>
              </Show>
              <AccountMenu me=me />
          </div>
      </div>
  }
}
