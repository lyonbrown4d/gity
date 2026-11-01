use super::model::UserPage;
use crate::{api, i18n::t, ui::*};
use leptos::prelude::*;
use serde_json::Value;

#[component]
pub(super) fn Directory(
    rows: Memo<UserPage>,
    loading: RwSignal<bool>,
    refresh: RwSignal<u64>,
    on_edit: Callback<Value>,
) -> impl IntoView {
    view! {
        <Show when=move || loading.get()>
            <div class="console-loading" aria-label="Loading">
                {(0..4).map(|_| view! { <span class="loading-placeholder"></span> }).collect_view()}
            </div>
        </Show>
        <Show when=move || !loading.get()>
            <div class="table-scroll">
                <table class="users-table">
                    <thead>
                        <tr>
                            {["Username", "Email", "Status", "Role", "Actions"]
                                .into_iter()
                                .map(|title| view! { <th>{t(title)}</th> })
                                .collect_view()}
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            rows.get()
                                .rows
                                .into_iter()
                                .map(|user| {
                                    let username = api::text(&user, "username");
                                    let edit = user.clone();
                                    let admin = user["is_super_admin"] == true;
                                    let status = api::text(&user, "status");
                                    let title = gity_i18n::translate(
                                        "Delete user \"{name}\"?",
                                        &crate::i18n::locale(),
                                        &[("name", &username)],
                                    );
                                    view! {
                                        <tr>
                                            <td>
                                                <div class="user-identity">
                                                    <i class:active=status.eq_ignore_ascii_case("active")></i>
                                                    <div>
                                                        <p>{username.clone()}</p>
                                                        <small>{api::text(&user, "id")}</small>
                                                    </div>
                                                </div>
                                            </td>
                                            <td class="user-email">{api::text(&user, "email")}</td>
                                            <td>
                                                <span class="badge">{status}</span>
                                            </td>
                                            <td>
                                                <span
                                                    class="badge"
                                                    class:primary=admin
                                                    class:outline=!admin
                                                >
                                                    {if admin { t("super-admin") } else { "user".to_string() }}
                                                </span>
                                            </td>
                                            <td>
                                                <div class="users-row-actions">
                                                    <button
                                                        type="button"
                                                        aria-label=format!("{} {username}", t("Edit"))
                                                        title=t("Edit")
                                                        on:click=move |_| on_edit.run(edit.clone())
                                                    >
                                                        <thaw::Icon
                                                            icon=icondata_lu::LuPencil
                                                            width="14px"
                                                            height="14px"
                                                        />
                                                    </button>
                                                    <Action
                                                        label=format!("{} {username}", t("Delete"))
                                                        path=format!("/users/{}", api::enc(&api::text(&user, "id")))
                                                        method="DELETE".into()
                                                        refresh=refresh
                                                        confirm=true
                                                        disabled=Signal::stored(admin)
                                                        confirmation_title=title
                                                        confirmation_description=t("This action cannot be undone.")
                                                    >
                                                        <thaw::Icon
                                                            icon=icondata_lu::LuTrash2
                                                            width="14px"
                                                            height="14px"
                                                        />
                                                    </Action>
                                                </div>
                                            </td>
                                        </tr>
                                    }
                                })
                                .collect_view()
                        }} <Show when=move || rows.get().rows.is_empty()>
                            <tr>
                                <td colspan="5" class="users-empty">
                                    {t("No users found.")}
                                </td>
                            </tr>
                        </Show>
                    </tbody>
                </table>
            </div>
        </Show>
    }
}
