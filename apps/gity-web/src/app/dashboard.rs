use crate::{api, ui::*};
use leptos::prelude::*;
#[component]
pub(super) fn Dashboard(#[prop(default = false)] admin: bool) -> impl IntoView {
    let user = query("/users/me".into());
    let orgs = query("/orgs".into());
    let projects = query(
        if admin {
            "/projects?all=true"
        } else {
            "/projects"
        }
        .into(),
    );
    let todos = query("/todos?state=pending".into());
    view! {
        <div class="dashboard">
            <section class="dashboard-summary">
                <div class="dashboard-summary-header">
                    <div>
                        <p class="workspace-path">
                            <span></span>
                            "Workspace / "
                            {move || api::text(&user.data.get(), "username")}
                        </p>
                        <h1>{crate::i18n::t("Dashboard")}</h1>
                        <p class="dashboard-description">
                            {crate::i18n::t(
                                "Project activity, organization scope, and account access in one workspace.",
                            )}
                        </p>
                    </div>
                    <div class="toolbar">
                        <a class="button primary" href="/app/projects">
                            {crate::i18n::t("Open Projects")}
                        </a>
                        <a class="button" href="/app/todos">
                            {crate::i18n::t("Open Inbox")}
                        </a>
                        <a class="button" href="/app/profile">
                            {crate::i18n::t("Edit Profile")}
                        </a>
                    </div>
                </div>
                <div class="dashboard-status">
                    <Fact
                        icon="org"
                        label="Organizations"
                        value=Signal::derive(move || api::list(&orgs.data.get()).len().to_string())
                        detail=Signal::stored("Memberships available in your scope.".to_string())
                    />
                    <Fact
                        icon="project"
                        label="Projects"
                        value=Signal::derive(move || {
                            api::list(&projects.data.get()).len().to_string()
                        })
                        detail=Signal::stored("Total projects visible to you.".to_string())
                    />
                    <Fact
                        icon="inbox"
                        label="Pending Todos"
                        value=Signal::derive(move || api::list(&todos.data.get()).len().to_string())
                        detail=Signal::stored("Project activity waiting in your inbox.".to_string())
                    />
                    <Fact
                        icon="user"
                        label="Account Status"
                        value=Signal::derive(move || api::text(&user.data.get(), "status"))
                        detail=Signal::derive(move || {
                            format!(
                                "{} / {}",
                                if user.data.get()["is_super_admin"] == true {
                                    "Super Admin"
                                } else {
                                    "User"
                                },
                                api::text(&user.data.get(), "email"),
                            )
                        })
                    />
                </div>
            </section>
            <Status q=user />
            <Status q=projects />
            <Status q=orgs />
            <div class="dashboard-columns">
                <section class="panel recent-projects">
                    <div class="panel-heading">
                        <div>
                            <h2>{crate::i18n::t("Recent Projects")}</h2>
                            <p class="muted">
                                {crate::i18n::t("Fast access to the repositories you can operate.")}
                            </p>
                        </div>
                        <a class="button ghost" href="/app/projects">
                            {crate::i18n::t("View all →")}
                        </a>
                    </div>
                    {move || {
                        api::list(&projects.data.get())
                            .into_iter()
                            .take(5)
                            .map(|p| {
                                view! {
                                    <a
                                        class="recent-project"
                                        href=format!(
                                            "/app/projects/{}/{}",
                                            api::text(&p, "organization_id"),
                                            api::text(&p, "id"),
                                        )
                                    >
                                        <thaw::Icon icon=icondata_lu::LuFolderGit2 />
                                        <span>
                                            <strong>{api::text(&p, "name")}</strong>
                                            <small>
                                                "⑂ "{api::text(&p, "key")}" / "
                                                {api::text(&p, "default_branch")}
                                            </small>
                                        </span>
                                        <span class="badge">{api::text(&p, "visibility")}</span>
                                    </a>
                                }
                            })
                            .collect_view()
                    }}
                    <Show when=move || {
                        !projects.loading.get() && api::list(&projects.data.get()).is_empty()
                    }>
                        <p class="muted list-empty">{crate::i18n::t("No projects available.")}</p>
                    </Show>
                </section>
                <section class="panel">
                    <div class="panel-heading">
                        <div>
                            <h2>{crate::i18n::t("My Organizations")}</h2>
                            <p class="muted">
                                {crate::i18n::t("Organizations you can operate in.")}
                            </p>
                        </div>
                    </div>
                    {move || {
                        api::list(&orgs.data.get())
                            .into_iter()
                            .map(|org| {
                                view! {
                                    <div class="organization-row">
                                        <div>
                                            <strong>{api::text(&org, "name")}</strong>
                                            <small>{api::text(&org, "key")}</small>
                                        </div>
                                        <span class="badge outline">{api::text(&org, "role")}</span>
                                    </div>
                                }
                            })
                            .collect_view()
                    }}
                    <Show when=move || {
                        !orgs.loading.get() && api::list(&orgs.data.get()).is_empty()
                    }>
                        <p class="muted list-empty">
                            {crate::i18n::t("No organizations available.")}
                        </p>
                    </Show>
                </section>
            </div>
        </div>
    }
}
#[component]
fn Fact(
    icon: &'static str,
    label: &'static str,
    value: Signal<String>,
    detail: Signal<String>,
) -> impl IntoView {
    let symbol = match icon {
        "org" => icondata_lu::LuBuilding2,
        "project" => icondata_lu::LuFolderGit2,
        "inbox" => icondata_lu::LuInbox,
        _ => icondata_lu::LuShieldCheck,
    };
    view! {
        <div class="dashboard-fact" role="group" aria-label=label>
            <span class="fact-icon">
                <thaw::Icon icon=symbol />
            </span>
            <div>
                <p>{crate::i18n::t(label)}</p>
                <strong>{move || value.get()}</strong>
                <small title=move || detail.get()>{move || detail.get()}</small>
            </div>
        </div>
    }
}
