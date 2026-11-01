mod collaboration;
mod comments;
mod markdown;
mod model;

use crate::{api, i18n::t, ui::*};
use collaboration::Collaboration;
use comments::Comments;
use leptos::prelude::*;
use markdown::IssueMarkdown;
use model::{IssueQueries, relative_time, user_label};
use serde_json::json;

#[component]
pub fn IssueDetail(project_path: String, number: String, base: String) -> impl IntoView {
    if !number.parse::<u64>().is_ok_and(|value| value > 0) {
        return view! {
            <p class="error" role="alert">
                {t("Invalid issue number")}
            </p>
        }
        .into_any();
    }
    let path = format!("{project_path}/issues/{}", api::enc(&number));
    let q = IssueQueries {
        issue: query(path.clone()),
        comments: query(format!("{path}/comments?limit=200")),
        assignees: query(format!("{path}/assignees")),
        labels: query(format!("{path}/labels")),
        users: query_all_pages("/users".into()),
        permissions: query(format!("{project_path}/permissions")),
    };
    let can_write = Signal::derive(move || {
        allowed(
            &q.permissions.data.get(),
            "project.issues.write",
            "issue_write",
            30,
        )
    });
    let can_comment = Signal::derive(move || {
        allowed(
            &q.permissions.data.get(),
            "project.issues.comment",
            "issue_comment",
            20,
        )
    });
    let description_base = StoredValue::new(base.clone());
    let comments_base = base.clone();
    let sidebar_path = path.clone();
    let comments_path = path.clone();
    let attachment_path = format!("{project_path}/issues/attachments?issue_number={number}");
    view! {
        <div class="issue-detail-page page-enter">
            <nav class="issue-detail-breadcrumb" aria-label=t("Breadcrumb")>
                <a href="/app/projects">{t("My Projects")}</a>
                <span>"/"</span>
                <a href=base.clone()>{t("Project")}</a>
                <span>"/"</span>
                <a href=format!("{base}?tab=issues")>{t("Issues")}</a>
            </nav>
            <Show when=move || q.issue.loading.get() && !q.issue.data.get().is_object()>
                <p class="muted" role="status">
                    {t("Loading issue...")}
                </p>
            </Show>
            <Show when=move || !q.issue.error.get().is_empty()>
                <p class="error" role="alert">
                    {move || q.issue.error.get()}
                </p>
            </Show>
            <Show when=move || !q.permissions.error.get().is_empty()>
                <p class="error" role="alert">
                    {move || q.permissions.error.get()}
                </p>
            </Show>
            <Show when=move || q.issue.data.get().is_object()>
                <Show when=move || !q.users.error.get().is_empty()>
                    <p class="error" role="alert">
                        {move || q.users.error.get()}
                    </p>
                </Show>
                <section class="issue-detail-card account-card card-enter">
                    <IssueHeader q=q path=path.clone() base=base.clone() can_write=can_write />
                    <div class="issue-detail-layout">
                        <main class="issue-detail-discussion">
                            <div class="issue-description">
                                {move || {
                                    let content = api::text(&q.issue.data.get(), "description");
                                    if content.is_empty() {
                                        view! {
                                            <p class="muted">{t("No description provided.")}</p>
                                        }
                                            .into_any()
                                    } else {
                                        view! {
                                            <IssueMarkdown
                                                content=content
                                                base=description_base.get_value()
                                            />
                                        }
                                            .into_any()
                                    }
                                }}
                            </div>
                            <Comments
                                q=q
                                path=comments_path.clone()
                                base=comments_base.clone()
                                can_comment=can_comment
                                attachment_path=attachment_path.clone()
                            />
                        </main>
                        <Collaboration q=q path=sidebar_path.clone() can_write=can_write />
                    </div>
                </section>
            </Show>
        </div>
    }
    .into_any()
}

#[component]
fn IssueHeader(
    q: IssueQueries,
    path: String,
    base: String,
    can_write: Signal<bool>,
) -> impl IntoView {
    let mutation = Mutation::new(q.issue.refresh);
    let back = format!("{base}?tab=issues");
    view! {
        <header class="issue-detail-header">
            <div>
                <div class="issue-detail-title">
                    <h1>
                        {move || {
                            format!(
                                "#{} {}",
                                api::text(&q.issue.data.get(), "number"),
                                api::text(&q.issue.data.get(), "title"),
                            )
                        }}
                    </h1>
                    <span
                        class="badge"
                        class:primary=move || q.issue.data.get()["status"] == "open"
                    >
                        {move || t(
                            if q.issue.data.get()["status"] == "open" { "Open" } else { "Closed" },
                        )}
                    </span>
                </div>
                <p>
                    {t("Opened by")}" "
                    {move || user_label(
                        &q.users.data.get(),
                        &api::text(&q.issue.data.get(), "author_user_id"),
                    )} " · "{move || relative_time(&api::text(&q.issue.data.get(), "created_at"))}
                    {move || {
                        let updated = api::text(&q.issue.data.get(), "updated_at");
                        if updated.is_empty() {
                            String::new()
                        } else {
                            format!(" · {} {}", t("updated"), relative_time(&updated))
                        }
                    }}
                </p>
            </div>
            <div class="issue-detail-header-actions">
                <button
                    disabled=move || {
                        !can_write.get() || mutation.busy.get() || q.issue.loading.get()
                    }
                    on:click=move |_| {
                        if can_write.get_untracked() {
                            let open = q.issue.data.get_untracked()["status"] == "open";
                            mutation
                                .run(
                                    "PATCH",
                                    path.clone(),
                                    json!({"status":if open { "closed" } else { "open" }}),
                                    None,
                                );
                        }
                    }
                >
                    {move || t(
                        if q.issue.data.get()["status"] == "open" {
                            "Close issue"
                        } else {
                            "Reopen issue"
                        },
                    )}
                </button>
                <a class="button" href=back>
                    {t("Back")}
                </a>
            </div>
            <Show when=move || !mutation.error.get().is_empty()>
                <p class="error" role="alert">
                    {move || mutation.error.get()}
                </p>
            </Show>
        </header>
    }
}
