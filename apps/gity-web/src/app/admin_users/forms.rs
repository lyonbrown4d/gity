use crate::{api, i18n::t, ui::*};
use leptos::prelude::*;
use serde_json::{Value, json};

#[component]
pub(super) fn UserForm(
    initial: Value,
    mutation: Mutation,
    on_cancel: Callback<()>,
    on_saved: Callback<()>,
) -> impl IntoView {
    let editing = initial.is_object();
    let username = RwSignal::new(api::text(&initial, "username"));
    let email = RwSignal::new(api::text(&initial, "email"));
    let password = RwSignal::new(String::new());
    let status = RwSignal::new(if initial["status"] == "disabled" {
        "disabled".to_string()
    } else {
        "active".to_string()
    });
    let role = RwSignal::new(if initial["is_super_admin"] == true {
        "super-admin".to_string()
    } else {
        "user".to_string()
    });
    let prefix = if editing { "edit-user" } else { "create-user" };
    let path = if editing {
        format!("/users/{}", api::enc(&api::text(&initial, "id")))
    } else {
        "/users".into()
    };
    let submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if mutation.busy.get_untracked() {
            return;
        }
        let mut body = json!({"username":username.get_untracked(),"email":email.get_untracked(),"is_super_admin":role.get_untracked()=="super-admin"});
        if editing {
            body["status"] = json!(status.get_untracked());
        }
        if !editing || !password.get_untracked().trim().is_empty() {
            body["password"] = json!(password.get_untracked());
        }
        mutation.run(
            if editing { "PATCH" } else { "POST" },
            path.clone(),
            body,
            Some(on_saved),
        );
    };
    view! {
        <form class="admin-user-form" on:submit=submit>
            <label for=format!(
                "{prefix}-username",
            )>
                {t("Username")}
                <input
                    id=format!("{prefix}-username")
                    required
                    autocomplete="username"
                    prop:value=move || username.get()
                    on:input=move |event| username.set(event_target_value(&event))
                />
            </label>
            <label for=format!(
                "{prefix}-email",
            )>
                {t("Email")}
                <input
                    id=format!("{prefix}-email")
                    type="email"
                    required
                    autocomplete="email"
                    prop:value=move || email.get()
                    on:input=move |event| email.set(event_target_value(&event))
                />
            </label>
            <label for=format!(
                "{prefix}-password",
            )>
                {t(if editing { "Password (optional)" } else { "Password" })}
                <input
                    id=format!("{prefix}-password")
                    type="password"
                    required=!editing
                    autocomplete="new-password"
                    prop:value=move || password.get()
                    on:input=move |event| password.set(event_target_value(&event))
                />
            </label>
            {editing
                .then(|| {
                    view! {
                        <label for="edit-user-status">
                            {t("Status")}
                            <select
                                id="edit-user-status"
                                aria-label=t("Status")
                                prop:value=move || status.get()
                                on:change=move |event| status.set(event_target_value(&event))
                            >
                                <option value="active">{t("active")}</option>
                                <option value="disabled">{t("disabled")}</option>
                            </select>
                        </label>
                    }
                })}
            <label for=format!(
                "{prefix}-role",
            )>
                {t("Privilege level")}
                <select
                    id=format!("{prefix}-role")
                    aria-label=t("Privilege level")
                    prop:value=move || role.get()
                    on:change=move |event| role.set(event_target_value(&event))
                >
                    <option value="user">{t("Regular user")}</option>
                    <option value="super-admin">{t("Super Admin")}</option>
                </select>
            </label>
            <Show when=move || !mutation.error.get().is_empty()>
                <p class="error" role="alert">
                    {move || mutation.error.get()}
                </p>
            </Show>
            <footer>
                <button type="button" on:click=move |_| on_cancel.run(())>
                    {t("Cancel")}
                </button>
                <button type="submit" class="primary" disabled=move || mutation.busy.get()>
                    {move || t(
                        if editing {
                            if mutation.busy.get() { "Saving..." } else { "Save" }
                        } else if mutation.busy.get() {
                            "Creating..."
                        } else {
                            "Create"
                        },
                    )}
                </button>
            </footer>
        </form>
    }
}
