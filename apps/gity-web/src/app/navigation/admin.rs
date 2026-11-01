use super::NavigationIcon;
use crate::{api, app::account::LogoutButton, i18n::t, ui::Query};
use leptos::prelude::*;

const ADMIN_LINKS: [(&str, &str); 4] = [
    ("/admin", "Overview"),
    ("/admin/orgs", "Organizations"),
    ("/admin/projects", "Projects"),
    ("/admin/users", "Users"),
];

#[component]
pub(super) fn AdminSidebar(path: String, me: Query) -> impl IntoView {
    view! {
        <div class="admin-brand">
            <span>{t("G_")}</span>
            <div>
                <strong>{t("Gity Console")}</strong>
                <small>{t("ADMIN SIDE")}</small>
            </div>
        </div>
        <nav class="admin-navigation" aria-label=t("Admin Side")>
            <p class="nav-label">{t("PLATFORM")}</p>
            {ADMIN_LINKS
                .into_iter()
                .map(|(url, label)| {
                    view! {
                        <a class:active=path == url href=url>
                            <NavigationIcon title=label.to_string() admin=true />
                            <span>{t(label)}</span>
                            <i class="admin-nav-active-dot" aria-hidden="true"></i>
                        </a>
                    }
                })
                .collect_view()}
        </nav>
        <div class="admin-identity">
            <thaw::Icon icon=icondata_lu::LuCircleUserRound width="16px" height="16px" />
            <div>
                <strong>{move || api::text(&me.data.get(), "username")}</strong>
                <small>{move || {
                    if me.data.get()["is_super_admin"] == true {
                        t("Super Admin")
                    } else {
                        t("Admin Side")
                    }
                }}</small>
            </div>
            <i aria-hidden="true"></i>
        </div>
        <LogoutButton />
    }
}

#[component]
pub(in crate::app) fn AdminMobileNavigation(path: String) -> impl IntoView {
    view! {
        <nav class="admin-mobile-navigation" aria-label=t("Admin Side")>
            {if path.starts_with("/admin") {
                ADMIN_LINKS
                    .into_iter()
                    .map(|(url, label)| {
                        view! {
                            <a class:active=path == url href=url>
                                <NavigationIcon title=label.to_string() admin=true />
                                {t(label)}
                            </a>
                        }
                    })
                    .collect_view()
                    .into_any()
            } else {
                ().into_any()
            }}
        </nav>
    }
}
