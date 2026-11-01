mod actions;
mod table;
use super::{
    admin_orgs::{
        model::ProjectRow, panels::LoadingRows, scope::DirectoryScope,
        selector::OrganizationSelector,
    },
    admin_ui::{ConsoleHeader, ConsoleMetric, ConsoleMetrics, ConsolePanel},
};
use crate::{i18n::t, ui::Mutation};
use leptos::prelude::*;

#[component]
pub(super) fn AdminProjects() -> impl IntoView {
    let scope = DirectoryScope::new();
    let selected = RwSignal::new(None::<ProjectRow>);
    let delete_open = RwSignal::new(false);
    let deletion = Mutation::new(scope.projects.refresh);
    let rows = scope.project_rows;
    let error = Signal::derive(move || {
        if deletion.error.get().is_empty() {
            scope.error.get()
        } else {
            deletion.error.get()
        }
    });
    let on_delete = Callback::new(move |project| {
        selected.set(Some(project));
        deletion.error.set(String::new());
        delete_open.set(true);
    });
    view! {
        <div class="admin-projects console-page page-enter">
            <ConsoleHeader
                title="Projects"
                description="Admin can view and delete projects. New project creation is handled in user workspace."
            >
                <span></span>
            </ConsoleHeader>
            <Show when=move || !error.get().is_empty()>
                <p class="error admin-directory-error" role="alert">
                    {move || error.get()}
                </p>
            </Show>
            <section class="admin-directory-scope project-admin-scope">
                <OrganizationSelector
                    id="admin-org-select"
                    organizations=scope.rows
                    selected=scope.selected
                />
                <div class="admin-directory-scope-badges">
                    <span class="badge">
                        {move || {
                            scope.current.get().map(|row| row.name).unwrap_or_else(|| t("N/A"))
                        }}
                    </span>
                    <span class="badge outline">
                        {move || scope.project_rows.get().len()}" "{t("projects")}
                    </span>
                </div>
            </section>
            {move || {
                view! {
                    <ConsoleMetrics items=vec![
                        ConsoleMetric {
                            label: "Organizations",
                            value: Signal::derive(move || scope.rows.get().len().to_string()),
                            detail: t("total"),
                            active: Signal::derive(move || !scope.rows.get().is_empty()),
                        },
                        ConsoleMetric {
                            label: "Projects",
                            value: Signal::derive(move || {
                                scope.project_rows.get().len().to_string()
                            }),
                            detail: t("projects"),
                            active: Signal::derive(move || !scope.project_rows.get().is_empty()),
                        },
                        ConsoleMetric {
                            label: "Organization",
                            value: Signal::derive(move || {
                                if scope.selected.get().is_empty() {
                                    "00".into()
                                } else {
                                    "01".into()
                                }
                            }),
                            detail: scope
                                .current
                                .get()
                                .map(|row| row.name)
                                .unwrap_or_else(|| t("N/A")),
                            active: Signal::derive(move || !scope.selected.get().is_empty()),
                        },
                    ] />
                }
            }}
            <ConsolePanel
                title="Project List"
                description="Projects under the selected organization."
                meta=Signal::derive(move || {
                    format!("{} {}", scope.project_rows.get().len(), t("projects"))
                })
            >
                <LoadingRows loading=scope.loading />
                <Show when=move || !scope.loading.get()>
                    <table::ProjectTable
                        rows=rows
                        on_delete=on_delete
                        busy=Signal::derive(move || deletion.busy.get())
                        error=error
                    />
                </Show>
            </ConsolePanel>
            <actions::DeleteProjectDialog open=delete_open selected=selected mutation=deletion />
        </div>
    }
}
