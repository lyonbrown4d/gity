use crate::{api, i18n::t, ui::*};
use leptos::prelude::*;
use serde_json::{Value, json};

mod protection;
use crate::ui::Mutation as BranchMutation;
use protection::Protection;

#[component]
pub(super) fn Branches(
    project_path: String,
    default_branch: Signal<String>,
    permissions: Query,
) -> impl IntoView {
    let q = query(format!("{project_path}/repository/branches"));
    let name = RwSignal::new(String::new());
    let mutation = BranchMutation::new(q.refresh);
    let creation = BranchMutation::new(q.refresh);
    let can_push = Signal::derive(move || {
        allowed(
            &permissions.data.get(),
            "project.repository.push",
            "repository_push",
            30,
        )
    });
    let can_admin = Signal::derive(move || {
        allowed(
            &permissions.data.get(),
            "project.repository.admin",
            "repository_admin",
            40,
        )
    });
    let create_path = format!("{project_path}/repository/branches");
    let submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let value = name.get_untracked().trim().to_string();
        if !can_push.get_untracked() || value.is_empty() || mutation.busy.get_untracked() {
            return;
        }
        creation.run(
            "POST",
            create_path.clone(),
            json!({"name":value}),
            Some(Callback::new(move |_| name.set(String::new()))),
        );
    };
    view! {
        <section class="repository-activity account-card card-enter">
            <header>
                <h2>{t("Branches")}</h2>
                <p>{t("Manage project branches and protections.")}</p>
            </header>
            <div class="repository-activity-body branch-list">
                <Show when=move || !permissions.loading.get() && !can_push.get()>
                    <p class="branch-notice">
                        {t(
                            "Your current project role can inspect branches, but cannot change them.",
                        )}
                    </p>
                </Show>
                <form class="branch-create" on:submit=submit>
                    <input
                        placeholder=t("New branch name")
                        aria-label=t("New branch name")
                        disabled=move || !can_push.get()
                        prop:value=move || name.get()
                        on:input=move |event| name.set(event_target_value(&event))
                    />
                    <button class="primary" disabled=move || !can_push.get() || creation.busy.get() || mutation.busy.get()>
                        {move || t(
                            if creation.busy.get() { "Creating..." } else { "Create branch" },
                        )}
                    </button>
                </form>
                <Show when=move || !creation.error.get().is_empty()>
                    <p class="error" role="alert">{move || creation.error.get()}</p>
                </Show>
                <Show when=move || !mutation.error.get().is_empty()>
                    <p class="error" role="alert">
                        {move || mutation.error.get()}
                    </p>
                </Show>
                <Show when=move || q.loading.get()>
                    <p class="muted" role="status">
                        {t("Loading branches...")}
                    </p>
                </Show>
                <Show when=move || !q.error.get().is_empty()>
                    <p class="error" role="alert">
                        {move || q.error.get()}
                    </p>
                </Show>
                {move || {
                    api::list(&q.data.get())
                        .into_iter()
                        .map(|branch| {
                            view! {
                                <BranchRow
                                    branch=branch
                                    project_path=project_path.clone()
                                    default_branch=default_branch
                                    can_admin=can_admin
                                    mutation=mutation
                                    refresh=q.refresh
                                />
                            }
                        })
                        .collect_view()
                }}
                <Show when=move || {
                    !q.loading.get() && q.error.get().is_empty()
                        && api::list(&q.data.get()).is_empty()
                }>
                    <p class="muted">{t("No branches found.")}</p>
                </Show>
            </div>
        </section>
    }
}

#[component]
fn BranchRow(
    branch: Value,
    project_path: String,
    default_branch: Signal<String>,
    can_admin: Signal<bool>,
    mutation: BranchMutation,
    refresh: RwSignal<u64>,
) -> impl IntoView {
    let name = api::text(&branch, "name");
    let protected = branch["is_protected"].as_bool().unwrap_or(false);
    let protection = branch["protection"].clone();
    let default_name = name.clone();
    let reported_default = branch["is_default"].as_bool().unwrap_or(false);
    let is_default =
        Signal::derive(move || reported_default || default_name == default_branch.get());
    let path = format!("{project_path}/repository/branches/{}", api::enc(&name));
    let protect_path = format!("{path}/{}", if protected { "unprotect" } else { "protect" });
    let sha = api::text(&branch, "last_commit_sha")
        .chars()
        .take(8)
        .collect::<String>();
    let title = gity_i18n::translate(
        "Delete branch \"{name}\"?",
        &crate::i18n::locale(),
        &[("name", &name)],
    );
    view! {
        <article class="repository-branch">
            <div>
                <div class="branch-identity">
                    <strong>{name.clone()}</strong>
                    <Show when=move || is_default.get()>
                        <span class="badge">{t("Default")}</span>
                    </Show>
                    <span class="badge" class:primary=protected class:outline=!protected>
                        {t(if protected { "Protected" } else { "Unprotected" })}
                    </span>
                </div>
                <p>{t("Last commit")}": "{if sha.is_empty() { t("N/A") } else { sha }}</p>
                {if protection.is_object() {
                    Some(
                        view! {
                            <Protection
                                value=protection
                                project_path=project_path
                                branch=name.clone()
                                can_admin=can_admin
                                mutation=mutation
                            />
                        },
                    )
                } else {
                    None
                }}
            </div>
            <div class="branch-actions">
                <button
                    disabled=move || !can_admin.get() || mutation.busy.get()
                    on:click=move |_| {
                        if can_admin.get_untracked() {
                            mutation.run("POST", protect_path.clone(), json!({}), None);
                        }
                    }
                >
                    {t(if protected { "Unprotect" } else { "Protect" })}
                </button>
                <Action
                    label="Delete".into()
                    path=path
                    method="DELETE".into()
                    body=json!({"confirmation":name})
                    refresh=refresh
                    confirm=true
                    confirmation_title=title
                    confirmation_description=t("This action cannot be undone.")
                    disabled=Signal::derive(move || {
                        !can_admin.get() || is_default.get() || mutation.busy.get()
                    })
                />
            </div>
        </article>
    }
}
