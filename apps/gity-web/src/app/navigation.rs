mod admin;
mod group;
mod rail;
use super::{account::Account, icons::NavigationIcon};
use crate::{api, ui::*};
pub(super) use admin::AdminMobileNavigation;
use admin::AdminSidebar;
use group::ContextGroup;
use leptos::prelude::*;
use rail::PrimaryRail;

#[component]
pub(super) fn Navigation(
    path: String,
    search: String,
    me: Query,
    collapsed: RwSignal<bool>,
    #[prop(default = false)] drawer: bool,
) -> impl IntoView {
    let projects = query_if(
        "/projects".into(),
        path.contains("projects") || path.contains("repositories"),
    );
    let admin = path.starts_with("/admin");
    let is_project = path.split('/').count() >= 5
        && (path.contains("/projects/") || path.contains("/repositories/"));
    let area = if admin {
        "admin"
    } else if path.contains("projects") || path.contains("repositories") {
        "projects"
    } else if path.contains("todos") {
        "inbox"
    } else {
        "workspace"
    };
    let route = path.split('/').take(5).collect::<Vec<_>>().join("/");
    let params = web_sys::UrlSearchParams::new_with_str(&search).ok();
    let current = params
        .as_ref()
        .and_then(|p| p.get("tab"))
        .unwrap_or("overview".into());
    let todo_state = if params.as_ref().and_then(|p| p.get("state")).as_deref() == Some("done") {
        "done"
    } else {
        "pending"
    };
    let heading = match area {
        "projects" => ("Code space", "Projects", "Code and collaboration"),
        "inbox" => ("Review queue", "Inbox", "Work requiring attention"),
        "admin" => ("Control plane", "Administration", "Platform operations"),
        _ => ("Personal space", "Workspace", "Your command center"),
    };
    let project_id = StoredValue::new(path.split('/').nth(4).unwrap_or("").to_string());
    let heading_projects = projects;
    view! {
        <aside
            class="sidebar"
            class:sidebar-collapsed=move || !drawer && collapsed.get()
            class:mobile-sidebar-content=drawer
        >
            {if admin {
                view! {
                    <AdminSidebar path=path.clone() me=me />
                }
                    .into_any()
            } else {
                view! {
                    <PrimaryRail me=me area=area />
                    <div class="context-panel">
                        <header>
                            <p>
                                {if is_project {
                                    crate::i18n::t("Project")
                                } else {
                                    crate::i18n::t(heading.0)
                                }}
                            </p>
                            <strong>
                                {move || {
                                    if is_project {
                                        api::list(&heading_projects.data.get())
                                            .into_iter()
                                            .find(|v| api::text(v, "id") == project_id.get_value())
                                            .map(|v| api::text(&v, "name"))
                                            .unwrap_or(heading.1.into())
                                    } else {
                                        crate::i18n::t(heading.1)
                                    }
                                }}
                            </strong>
                            <small title=move || {
                                api::list(&heading_projects.data.get())
                                    .into_iter()
                                    .find(|v| api::text(v, "id") == project_id.get_value())
                                    .map(|v| api::text(&v, "full_path"))
                                    .unwrap_or_default()
                            }>
                                {move || {
                                    if is_project {
                                        api::list(&heading_projects.data.get())
                                            .into_iter()
                                            .find(|v| api::text(v, "id") == project_id.get_value())
                                            .map(|v| api::text(&v, "full_path"))
                                            .unwrap_or_default()
                                    } else {
                                        crate::i18n::t(heading.2)
                                    }
                                }}
                            </small>
                        </header>
                        <div class="context-content">
                            {if is_project {
                                view! {
                                    <ContextGroup
                                        label="Projects".into()
                                        links=vec![
                                            ("Back to projects".into(), "/app/projects".into()),
                                        ]
                                        active=String::new()
                                    />
                                    {[
                                        (
                                            "Project",
                                            vec![
                                                ("overview", "Overview"),
                                                ("code", "Code"),
                                                ("wiki", "Wiki"),
                                                ("packages", "Packages"),
                                                ("releases", "Releases"),
                                            ],
                                        ),
                                        (
                                            "Plan",
                                            vec![
                                                ("issues", "Issues"),
                                                ("merge-requests", "Merge Requests"),
                                            ],
                                        ),
                                        (
                                            "Repository",
                                            vec![
                                                ("commits", "Commits"),
                                                ("branches", "Branches"),
                                                ("lfs", "LFS"),
                                            ],
                                        ),
                                        (
                                            "CI/CD",
                                            vec![
                                                ("pipelines", "Pipelines"),
                                                ("jobs", "Jobs"),
                                                ("runners", "Runners"),
                                            ],
                                        ),
                                        (
                                            "Operate",
                                            vec![("audit", "Audit"), ("settings", "Settings")],
                                        ),
                                    ]
                                        .into_iter()
                                        .map(|(label, items)| {
                                            let active = format!("{route}?tab={current}");
                                            view! {
                                                <ContextGroup
                                                    label=label.to_string()
                                                    collapsible=true
                                                    links=items
                                                        .into_iter()
                                                        .map(|(tab, title)| (
                                                            title.to_string(),
                                                            format!("{route}?tab={tab}"),
                                                        ))
                                                        .collect()
                                                    active=active
                                                />
                                            }
                                        })
                                        .collect_view()}
                                }
                                    .into_any()
                            } else if area == "projects" {
                                view! {
                                    <ContextGroup
                                        label="Projects".into()
                                        links=vec![("All projects".into(), "/app/projects".into())]
                                        active=path.clone()
                                    />
                                    {move || {
                                        view! {
                                            <ContextGroup
                                                label="Recent".into()
                                                links=api::list(&projects.data.get())
                                                    .into_iter()
                                                    .take(6)
                                                    .map(|p| (
                                                        api::text(&p, "name"),
                                                        format!(
                                                            "/app/projects/{}/{}",
                                                            api::text(&p, "organization_id"),
                                                            api::text(&p, "id"),
                                                        ),
                                                    ))
                                                    .collect()
                                                active=String::new()
                                            />
                                        }
                                    }}
                                }
                                    .into_any()
                            } else if area == "inbox" {
                                view! {
                                    <ContextGroup
                                        label="Queues".into()
                                        links=vec![
                                            ("Pending".into(), "/app/todos?state=pending".into()),
                                            ("Done".into(), "/app/todos?state=done".into()),
                                        ]
                                        active=format!("/app/todos?state={todo_state}")
                                    />
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <ContextGroup
                                        label="Workspace".into()
                                        links=vec![
                                            ("Dashboard".into(), "/app/dashboard".into()),
                                            ("Profile".into(), "/app/profile".into()),
                                        ]
                                        active=path.clone()
                                    />
                                }
                                    .into_any()
                            }}
                        </div>
                        <Account me=me />
                    </div>
                }
                    .into_any()
            }}
        </aside>
    }
}
