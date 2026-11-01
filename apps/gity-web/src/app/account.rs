use crate::{api, i18n::t, ui::Query};
use leptos::prelude::*;
use thaw::MenuTrigger;

#[component]
pub(super) fn Account(me: Query, #[prop(default = false)] menu: bool) -> impl IntoView {
    view! {
        <footer class="context-account">
            {if menu {
                view! { <AccountMenu me=me compact=false /> }.into_any()
            } else {
                view! {
                    <div>
                        <strong>{move || api::text(&me.data.get(), "username")}</strong>
                        <small>{move || api::text(&me.data.get(), "email")}</small>
                    </div>
                }
                    .into_any()
            }}
        </footer>
    }
}

#[component]
pub(super) fn AccountMenu(me: Query, #[prop(default = true)] compact: bool) -> impl IntoView {
    let initials = Signal::derive(move || {
        let user = api::text(&me.data.get(), "username");
        let letters = user
            .split_whitespace()
            .take(2)
            .filter_map(|part| part.chars().next())
            .collect::<String>()
            .to_uppercase();
        if letters.is_empty() {
            "GU".into()
        } else {
            letters
        }
    });
    view! {
        <thaw::Menu
            class="account-menu"
            position=thaw::MenuPosition::RightEnd
            on_select=move |value: &'static str| {
                if value == "logout" {
                    logout();
                } else {
                    api::navigate(value);
                }
            }
        >
            <MenuTrigger slot>
                <button
                    class="account-avatar"
                    class:expanded=!compact
                    aria-label=move || {
                        format!("{}: {}", t("My Account"), api::text(&me.data.get(), "username"))
                    }
                >
                    <span>{move || initials.get()}</span>
                    {if compact {
                        None
                    } else {
                        Some(
                            view! {
                                <div>
                                    <strong>{move || api::text(&me.data.get(), "username")}</strong>
                                    <small>{move || api::text(&me.data.get(), "email")}</small>
                                </div>
                            },
                        )
                    }}
                </button>
            </MenuTrigger>
            <div class="account-menu-heading">
                <span>{move || initials.get()}</span>
                <div>
                    <strong>{move || api::text(&me.data.get(), "username")}</strong>
                    <small>{move || api::text(&me.data.get(), "email")}</small>
                </div>
            </div>
            <hr />
            <thaw::MenuItem value="/app/profile" icon=icondata_lu::LuUserRound>
                {t("Profile")}
            </thaw::MenuItem>
            <Show when=move || me.data.get()["is_super_admin"] == true>
                <thaw::MenuItem value="/admin" icon=icondata_lu::LuShield>
                    {t("Admin Dashboard")}
                </thaw::MenuItem>
            </Show>
            <hr />
            <thaw::MenuItem value="logout" icon=icondata_lu::LuLogOut>
                {t("Logout")}
            </thaw::MenuItem>
        </thaw::Menu>
    }
}

#[component]
pub(super) fn LogoutButton() -> impl IntoView {
    view! {
        <button class="admin-sidebar-logout" type="button" on:click=move |_| logout()>
            <thaw::Icon icon=icondata_lu::LuLogOut width="14px" height="14px" />
            {t("Logout")}
        </button>
    }
}

pub(super) fn logout() {
    wasm_bindgen_futures::spawn_local(async move {
        let _ = api::request(
            "POST",
            "/auth/logout",
            Some(serde_json::json!({"refresh_token":api::storage("gity.refresh_token")})),
        )
        .await;
        api::clear_auth();
        api::navigate("/login");
    });
}
