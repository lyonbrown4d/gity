use super::model::{self, OrganizationRow, ProjectRow};
use crate::{api, ui::*};
use leptos::prelude::*;

#[derive(Clone, Copy)]
pub(crate) struct DirectoryScope {
    pub organizations: Query,
    pub projects: Query,
    pub selected: RwSignal<String>,
    pub rows: Memo<Vec<OrganizationRow>>,
    pub current: Memo<Option<OrganizationRow>>,
    pub project_rows: Memo<Vec<ProjectRow>>,
    pub loading: Signal<bool>,
    pub error: Signal<String>,
}
impl DirectoryScope {
    pub fn new() -> Self {
        let organizations = query("/orgs".into());
        let selected = RwSignal::new(String::new());
        let rows = Memo::new(move |_| model::organizations(&organizations.data.get()));
        Effect::new(move |_| {
            let rows = rows.get();
            if organizations.loading.get() || !organizations.error.get().is_empty() {
                return;
            }
            let current = selected.get_untracked();
            if !rows.iter().any(|row| row.id == current) {
                selected.set(rows.first().map(|row| row.id.clone()).unwrap_or_default());
            }
        });
        let projects = query_fn(
            move || {
                if selected.get().is_empty() {
                    String::new()
                } else {
                    format!("/projects?organization_id={}", api::enc(&selected.get()))
                }
            },
            true,
        );
        Self {
            organizations,
            projects,
            selected,
            rows,
            current: Memo::new(move |_| {
                rows.get().into_iter().find(|row| row.id == selected.get())
            }),
            project_rows: Memo::new(move |_| {
                if selected.get().is_empty() || !projects.error.get().is_empty() {
                    Vec::new()
                } else {
                    model::projects_in_organization(
                        &model::projects(&projects.data.get()),
                        &selected.get(),
                    )
                }
            }),
            loading: Signal::derive(move || organizations.loading.get() || projects.loading.get()),
            error: Signal::derive(move || {
                [organizations.error.get(), projects.error.get()]
                    .into_iter()
                    .find(|error| !error.is_empty())
                    .unwrap_or_default()
            }),
        }
    }
}
