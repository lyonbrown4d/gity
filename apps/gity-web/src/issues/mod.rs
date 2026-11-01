use crate::{api, ui::*};
use leptos::prelude::*;
use std::collections::BTreeSet;
mod composer;
mod filters;
use filters::{assignees, labels};
mod detail;
mod row;
use composer::IssueComposer;
pub use detail::IssueDetail;
use row::IssueRow;
#[component]
pub fn IssueBoard(path: String, base: String) -> impl IntoView {
    let q = query(format!("{path}?status=all&limit=200"));
    let permissions = query(format!("{}/permissions", path.trim_end_matches("/issues")));
    let can_create = Signal::derive(move || {
        allowed(
            &permissions.data.get(),
            "project.issues.create",
            "issue_create",
            20,
        )
    });
    let users = query_all_pages("/users".into());
    let status = RwSignal::new("open".to_string());
    let term = RwSignal::new(String::new());
    let label = RwSignal::new("all".to_string());
    let assignee = RwSignal::new("all".to_string());
    let sort = RwSignal::new("updated_desc".to_string());
    let composer = RwSignal::new(false);
    let visible = Signal::derive(move || {
        let mut rows = api::list(&q.data.get())
            .into_iter()
            .filter(|r| status.get() == "all" || api::text(r, "status") == status.get())
            .filter(|r| {
                r.to_string()
                    .to_lowercase()
                    .contains(&term.get().to_lowercase())
            })
            .filter(|r| match label.get().as_str() {
                "all" => true,
                "unlabeled" => labels(r).is_empty(),
                wanted => labels(r).iter().any(|l| l == wanted),
            })
            .filter(|r| match assignee.get().as_str() {
                "all" => true,
                "unassigned" => assignees(r).is_empty(),
                wanted => assignees(r).iter().any(|id| id == wanted),
            })
            .collect::<Vec<_>>();
        let mode = sort.get();
        rows.sort_by(|a, b| {
            if mode.starts_with("number") {
                let order = a["number"]
                    .as_u64()
                    .unwrap_or(0)
                    .cmp(&b["number"].as_u64().unwrap_or(0));
                if mode == "number_asc" {
                    order
                } else {
                    order.reverse()
                }
            } else {
                api::text(
                    b,
                    if mode == "created_desc" {
                        "created_at"
                    } else {
                        "updated_at"
                    },
                )
                .cmp(&api::text(
                    a,
                    if mode == "created_desc" {
                        "created_at"
                    } else {
                        "updated_at"
                    },
                ))
            }
        });
        rows
    });
    let reset = move |_| {
        status.set("open".into());
        term.set(String::new());
        label.set("all".into());
        assignee.set("all".into());
        sort.set("updated_desc".into());
    };
    let create_base = base.clone();
    view! {
        <section class="issue-board panel">
            <header>
                <h2>{crate::i18n::t("Issues")}</h2>
                <p>
                    {crate::i18n::t(
                        "Track bugs, tasks, ownership, labels, and discussions for this project.",
                    )}
                </p>
            </header>
            <div class="issue-board-body">
                <div class="issue-stat-grid">
                    {["Open", "Closed", "Unassigned", "Labeled", "Total"]
                        .into_iter()
                        .map(|name| {
                            view! {
                                <div>
                                    <small>{crate::i18n::t(name)}</small>
                                    <strong>
                                        {move || {
                                            api::list(&q.data.get())
                                                .iter()
                                                .filter(|r| match name {
                                                    "Open" => r["status"] == "open",
                                                    "Closed" => r["status"] == "closed",
                                                    "Unassigned" => assignees(r).is_empty(),
                                                    "Labeled" => !labels(r).is_empty(),
                                                    _ => true,
                                                })
                                                .count()
                                        }}
                                    </strong>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
                <div class="issue-filter-box">
                    <div class="issue-filters">
                        <input
                            placeholder="Search title, label, or assignee"
                            aria-label="Search issues"
                            prop:value=move || term.get()
                            on:input=move |e| term.set(event_target_value(&e))
                        />
                        <select
                            aria-label="Issue status"
                            prop:value=move || status.get()
                            on:change=move |e| status.set(event_target_value(&e))
                        >
                            <option value="open">{crate::i18n::t("Open issues")}</option>
                            <option value="closed">{crate::i18n::t("Closed issues")}</option>
                            <option value="all">{crate::i18n::t("All issues")}</option>
                        </select>
                        <select
                            aria-label="Label"
                            prop:value=move || label.get()
                            on:change=move |e| label.set(event_target_value(&e))
                        >
                            <option value="all">{crate::i18n::t("Any label")}</option>
                            <option value="unlabeled">{crate::i18n::t("Unlabeled")}</option>
                            {move || {
                                api::list(&q.data.get())
                                    .iter()
                                    .flat_map(labels)
                                    .collect::<BTreeSet<_>>()
                                    .into_iter()
                                    .map(|label| {
                                        view! {
                                            <option value=label.clone()>{label.clone()}</option>
                                        }
                                    })
                                    .collect_view()
                            }}
                        </select>
                        <select
                            aria-label="Assignee"
                            prop:value=move || assignee.get()
                            on:change=move |e| assignee.set(event_target_value(&e))
                        >
                            <option value="all">{crate::i18n::t("Any assignee")}</option>
                            <option value="unassigned">{crate::i18n::t("Unassigned")}</option>
                            {move || {
                                api::list(&users.data.get())
                                    .into_iter()
                                    .map(|u| {
                                        view! {
                                            <option value=api::text(
                                                &u,
                                                "id",
                                            )>{api::text(&u, "username")}</option>
                                        }
                                    })
                                    .collect_view()
                            }}
                        </select>
                        <select
                            aria-label="Sort issues"
                            prop:value=move || sort.get()
                            on:change=move |e| sort.set(event_target_value(&e))
                        >
                            <option value="updated_desc">
                                {crate::i18n::t("Recently updated")}
                            </option>
                            <option value="created_desc">
                                {crate::i18n::t("Recently created")}
                            </option>
                            <option value="number_desc">{crate::i18n::t("Newest number")}</option>
                            <option value="number_asc">{crate::i18n::t("Oldest number")}</option>
                        </select>
                    </div>
                    <div class="issue-filter-summary">
                        <small>
                            "Showing "{move || visible.get().len()}" of "
                            {move || api::list(&q.data.get()).len()}" issues"
                        </small>
                        <button on:click=reset>{crate::i18n::t("Clear filters")}</button>
                    </div>
                </div>
                <div class="issue-board-actions">
                    <thaw::Button
                        disabled=Signal::derive(move || !can_create.get())
                        on_click=move |_| { composer.set(!composer.get_untracked()) }
                    >
                        {move || {
                            if composer.get() {
                                crate::i18n::t("Hide new issue form")
                            } else {
                                crate::i18n::t("New issue")
                            }
                        }}
                    </thaw::Button>
                    <thaw::Button
                        appearance=thaw::ButtonAppearance::Subtle
                        on_click=move |_| q.refresh.update(|v| *v += 1)
                    >
                        {crate::i18n::t("Reload")}
                    </thaw::Button>
                </div>
                <Show when=move || !permissions.loading.get() && !can_create.get()>
                    <p class="branch-notice">
                        {crate::i18n::t(
                            "Your current project role can inspect issues, but cannot create them.",
                        )}
                    </p>
                </Show>
                <Show when=move || composer.get() && can_create.get()>
                    <IssueComposer path=path.clone() base=create_base.clone() users=users />
                </Show>
                <Status q=q />
                <Show when=move || !users.error.get().is_empty()>
                    <p class="error" role="alert">
                        {move || users.error.get()}
                    </p>
                </Show>
                <div class="issue-list">
                    {move || {
                        visible
                            .get()
                            .into_iter()
                            .map(|issue| {
                                view! { <IssueRow issue=issue base=base.clone() users=users /> }
                            })
                            .collect_view()
                    }}<Show when=move || !q.loading.get() && visible.get().is_empty()>
                        <div class="empty">
                            <h3>
                                {move || {
                                    if api::list(&q.data.get()).is_empty() {
                                        "No issues yet."
                                    } else {
                                        "No issues match these filters."
                                    }
                                }}
                            </h3>
                            <p>
                                {crate::i18n::t(
                                    "Create the first issue to start triage for this project.",
                                )}
                            </p>
                            <button
                                disabled=move || !can_create.get()
                                on:click=move |_| { composer.set(true) }
                            >
                                {crate::i18n::t("Create an issue")}
                            </button>
                        </div>
                    </Show>
                </div>
            </div>
        </section>
    }
}
