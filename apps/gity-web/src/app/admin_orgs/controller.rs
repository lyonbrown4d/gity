use super::model::{self, MemberRow, OrganizationRow, ProjectRow};
use crate::{api, ui::*};
use leptos::prelude::*;

#[derive(Clone, Copy)]
pub(crate) struct OrganizationAdmin {
    pub selected: RwSignal<String>,
    pub rows: Memo<Vec<OrganizationRow>>,
    pub current: Memo<Option<OrganizationRow>>,
    pub member_rows: Memo<Vec<MemberRow>>,
    pub project_rows: Memo<Vec<ProjectRow>>,
    pub create_open: RwSignal<bool>,
    pub edit_open: RwSignal<bool>,
    pub member_open: RwSignal<bool>,
    pub delete_open: RwSignal<bool>,
    pub creation: Mutation,
    pub update: Mutation,
    pub member: Mutation,
    pub deletion: Mutation,
    pub loading: Signal<bool>,
    pub error: Signal<String>,
    pub can_manage: Signal<bool>,
}
impl OrganizationAdmin {
    pub fn new() -> Self {
        let scope = super::scope::DirectoryScope::new();
        let organizations = scope.organizations;
        let selected = scope.selected;
        let rows = scope.rows;
        let current = scope.current;
        let projects = scope.projects;
        let project_rows = scope.project_rows;
        let members = query_fn(
            move || {
                if selected.get().is_empty() {
                    String::new()
                } else {
                    format!("/orgs/{}/members", api::enc(&selected.get()))
                }
            },
            true,
        );
        let member_rows = Memo::new(move |_| {
            if selected.get().is_empty() || !members.error.get().is_empty() {
                Vec::new()
            } else {
                model::members(&members.data.get())
            }
        });
        let creation = Mutation::new(organizations.refresh);
        let update = Mutation::new(organizations.refresh);
        let member = Mutation::new(members.refresh);
        let deletion = Mutation::new(organizations.refresh);
        Self {
            selected,
            rows,
            current,
            member_rows,
            project_rows,
            create_open: RwSignal::new(false),
            edit_open: RwSignal::new(false),
            member_open: RwSignal::new(false),
            delete_open: RwSignal::new(false),
            creation,
            update,
            member,
            deletion,
            loading: Signal::derive(move || {
                organizations.loading.get() || members.loading.get() || projects.loading.get()
            }),
            error: Signal::derive(move || {
                [
                    creation.error.get(),
                    update.error.get(),
                    member.error.get(),
                    deletion.error.get(),
                    organizations.error.get(),
                    members.error.get(),
                    projects.error.get(),
                ]
                .into_iter()
                .find(|error| !error.is_empty())
                .unwrap_or_default()
            }),
            can_manage: Signal::derive(move || {
                !organizations.loading.get()
                    && organizations.error.get().is_empty()
                    && current.get().is_some_and(|row| row.role == "owner")
            }),
        }
    }
    pub fn clear_errors(self) {
        for mutation in [self.creation, self.update, self.member, self.deletion] {
            mutation.error.set(String::new());
        }
    }
}
