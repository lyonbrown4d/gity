use super::super::account::AccountMenu;
use crate::ui::Query;
use leptos::prelude::*;

#[component]
pub(super) fn PrimaryRail(me: Query, area: &'static str) -> impl IntoView {
    view! {
        <div class="primary-rail">
            <a class="rail-brand" aria-label="Gity" href="/app/dashboard">
                <thaw::Icon icon=icondata_lu::LuArrowRightLeft width="16px" height="16px" />
            </a>
            <nav aria-label="Primary navigation">
                <a
                    title="Workspace"
                    aria-label="Workspace"
                    class:active=area == "workspace"
                    href="/app/dashboard"
                >
                    <thaw::Icon icon=icondata_lu::LuHouse />
                    <small>{crate::i18n::t("Home")}</small>
                </a>
                <a
                    title="Projects"
                    aria-label="Projects"
                    class:active=area == "projects"
                    href="/app/projects"
                >
                    <thaw::Icon icon=icondata_lu::LuFolderGit2 />
                    <small>{crate::i18n::t("Projects")}</small>
                </a>
                <a title="Inbox" aria-label="Inbox" class:active=area == "inbox" href="/app/todos">
                    <thaw::Icon icon=icondata_lu::LuInbox />
                    <small>{crate::i18n::t("Inbox")}</small>
                </a>
            </nav>
            <div class="rail-footer">
                <Show when=move || me.data.get()["is_super_admin"] == true>
                    <a title="Admin" aria-label="Admin" href="/admin">
                        <thaw::Icon icon=icondata_lu::LuShieldCheck />
                        <small>{crate::i18n::t("Admin")}</small>
                    </a>
                </Show>
                <AccountMenu me=me />
            </div>
        </div>
    }
}
