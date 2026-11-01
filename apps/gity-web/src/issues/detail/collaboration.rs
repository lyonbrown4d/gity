use super::model::{IssueQueries, labels_body, relative_time, user_label};
use crate::{api, i18n::t, ui::*};
use leptos::prelude::*;
use serde_json::json;

#[component]
pub(super) fn Collaboration(
    q: IssueQueries,
    path: String,
    can_write: Signal<bool>,
) -> impl IntoView {
    let assignee = RwSignal::new(String::new());
    let label_name = RwSignal::new(String::new());
    let label_color = RwSignal::new("#2563eb".to_string());
    let assignees = Mutation::new(q.assignees.refresh);
    let labels = Mutation::new(q.labels.refresh);
    let assignees_path = format!("{path}/assignees");
    let labels_path = format!("{path}/labels");
    let assign = Callback::new(move |ids: Vec<String>| {
        if can_write.get_untracked() {
            assignees.run(
                "PATCH",
                assignees_path.clone(),
                json!({"user_ids":ids}),
                Some(Callback::new(move |_| assignee.set(String::new()))),
            );
        }
    });
    let assign_selected = move |_| {
        let id = assignee.get_untracked();
        if id.is_empty() {
            return;
        }
        let mut ids = api::list(&q.assignees.data.get_untracked())
            .iter()
            .map(|row| api::text(row, "user_id"))
            .collect::<Vec<_>>();
        if !ids.contains(&id) {
            ids.push(id);
        }
        assign.run(ids);
    };
    let update_labels = Callback::new(
        move |(remove, addition): (Option<String>, Option<(String, String)>)| {
            if !can_write.get_untracked() {
                return;
            }
            let body = labels_body(
                api::list(&q.labels.data.get_untracked()),
                remove.as_deref(),
                addition
                    .as_ref()
                    .map(|(name, color)| (name.as_str(), color.as_str())),
            );
            labels.run(
                "PATCH",
                labels_path.clone(),
                body,
                Some(Callback::new(move |_| label_name.set(String::new()))),
            );
        },
    );
    let loading = Signal::derive(move || {
        q.assignees.loading.get() || q.labels.loading.get() || q.users.loading.get()
    });
    view! {
        <aside class="issue-collaboration">
            <header>
                <div>
                    <p>{t("Issue sidebar")}</p>
                    <small>{t("GitLab-style triage fields for this issue.")}</small>
                </div>
                <button
                    disabled=move || loading.get()
                    on:click=move |_| {
                        q.assignees.refresh.update(|n| *n += 1);
                        q.labels.refresh.update(|n| *n += 1);
                    }
                >
                    {move || t(if loading.get() { "Loading..." } else { "Reload" })}
                </button>
            </header>
            <SidebarSection title="Status">
                <div class="issue-sidebar-tags">
                    <span
                        class="badge"
                        class:primary=move || q.issue.data.get()["status"] == "open"
                    >
                        {move || t(
                            if q.issue.data.get()["status"] == "open" { "Open" } else { "Closed" },
                        )}
                    </span>
                    {move || {
                        let closed = api::text(&q.issue.data.get(), "closed_at");
                        (!closed.is_empty())
                            .then(|| {
                                view! { <small>{t("Closed")}" "{relative_time(&closed)}</small> }
                            })
                    }}
                </div>
            </SidebarSection>
            <SidebarSection title="Assignees">
                <div class="issue-sidebar-tags">
                    <Show when=move || api::list(&q.assignees.data.get()).is_empty()>
                        <small>{t("No assignees assigned.")}</small>
                    </Show>
                    {move || {
                        api::list(&q.assignees.data.get())
                            .into_iter()
                            .map(|row| {
                                let id = api::text(&row, "user_id");
                                let remove = id.clone();
                                let label = user_label(&q.users.data.get(), &id);
                                view! {
                                    <span class="badge outline">
                                        {label.clone()}
                                        <button
                                            type="button"
                                            aria-label=format!("Remove {label}")
                                            disabled=move || !can_write.get() || assignees.busy.get()
                                            on:click=move |_| {
                                                assign
                                                    .run(
                                                        api::list(&q.assignees.data.get_untracked())
                                                            .iter()
                                                            .map(|row| api::text(row, "user_id"))
                                                            .filter(|id| id != &remove)
                                                            .collect(),
                                                    )
                                            }
                                        >
                                            "x"
                                        </button>
                                    </span>
                                }
                            })
                            .collect_view()
                    }}
                </div>
                <div class="issue-sidebar-add">
                    <select
                        aria-label=t("Select user")
                        disabled=move || !can_write.get() || assignees.busy.get()
                        prop:value=move || assignee.get()
                        on:change=move |event| assignee.set(event_target_value(&event))
                    >
                        <option value="">{t("Select user")}</option>
                        {move || {
                            api::list(&q.users.data.get())
                                .into_iter()
                                .filter(|user| {
                                    !api::list(&q.assignees.data.get())
                                        .iter()
                                        .any(|row| {
                                            api::text(row, "user_id") == api::text(user, "id")
                                        })
                                })
                                .map(|user| {
                                    let id = api::text(&user, "id");
                                    view! {
                                        <option value=id
                                            .clone()>{user_label(&q.users.data.get(), &id)}</option>
                                    }
                                })
                                .collect_view()
                        }}
                    </select>
                    <button
                        type="button"
                        disabled=move || {
                            !can_write.get() || assignees.busy.get() || assignee.get().is_empty()
                        }
                        on:click=assign_selected
                    >
                        {move || t(if assignees.busy.get() { "Saving..." } else { "Add" })}
                    </button>
                </div>
                <Show when=move || !q.assignees.error.get().is_empty()>
                    <p class="error" role="alert">
                        {move || q.assignees.error.get()}
                    </p>
                </Show>
                <Show when=move || !assignees.error.get().is_empty()>
                    <p class="error" role="alert">
                        {move || assignees.error.get()}
                    </p>
                </Show>
            </SidebarSection>
            <SidebarSection title="Labels">
                <div class="issue-sidebar-tags">
                    <Show when=move || api::list(&q.labels.data.get()).is_empty()>
                        <small>{t("No labels assigned.")}</small>
                    </Show>
                    {move || {
                        api::list(&q.labels.data.get())
                            .into_iter()
                            .map(|row| {
                                let name = api::text(&row, "name");
                                let remove = name.clone();
                                let color = api::text(&row, "color");
                                view! {
                                    <span
                                        class="badge outline"
                                        style:color=color.clone()
                                        style:border-color=color
                                    >
                                        {name.clone()}
                                        <button
                                            type="button"
                                            aria-label=format!("Remove {name}")
                                            disabled=move || !can_write.get() || labels.busy.get()
                                            on:click=move |_| {
                                                update_labels.run((Some(remove.clone()), None))
                                            }
                                        >
                                            "x"
                                        </button>
                                    </span>
                                }
                            })
                            .collect_view()
                    }}
                </div>
                <input
                    aria-label=t("Label name")
                    placeholder=t("Label name")
                    disabled=move || !can_write.get()
                    prop:value=move || label_name.get()
                    on:input=move |event| label_name.set(event_target_value(&event))
                />
                <div class="issue-sidebar-add">
                    <input
                        aria-label=t("Label color")
                        placeholder="#2563eb"
                        disabled=move || !can_write.get()
                        prop:value=move || label_color.get()
                        on:input=move |event| label_color.set(event_target_value(&event))
                    />
                    <button
                        type="button"
                        disabled=move || {
                            !can_write.get() || labels.busy.get()
                                || label_name.get().trim().is_empty()
                        }
                        on:click=move |_| {
                            update_labels
                                .run((
                                    None,
                                    Some((label_name.get_untracked(), label_color.get_untracked())),
                                ))
                        }
                    >
                        {move || t(if labels.busy.get() { "Saving..." } else { "Add" })}
                    </button>
                </div>
                <Show when=move || !q.labels.error.get().is_empty()>
                    <p class="error" role="alert">
                        {move || q.labels.error.get()}
                    </p>
                </Show>
                <Show when=move || !labels.error.get().is_empty()>
                    <p class="error" role="alert">
                        {move || labels.error.get()}
                    </p>
                </Show>
            </SidebarSection>
        </aside>
    }
}

#[component]
fn SidebarSection(title: &'static str, children: Children) -> impl IntoView {
    view! {
        <section class="issue-sidebar-section">
            <h3>{t(title)}</h3>
            {children()}
        </section>
    }
}
