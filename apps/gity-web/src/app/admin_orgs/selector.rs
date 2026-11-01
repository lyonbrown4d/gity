use super::{controller::OrganizationAdmin, model::OrganizationRow};
use crate::i18n::t;
use leptos::prelude::*;

#[component]
pub(crate) fn OrganizationSelector(
    id: &'static str,
    organizations: Memo<Vec<OrganizationRow>>,
    selected: RwSignal<String>,
) -> impl IntoView {
    view! {
        <label class="admin-directory-org-select" for=id>
            {t("Organization")}
            <select
                id=id
                aria-label=t("Organization")
                prop:value=move || selected.get()
                on:change=move |event| selected.set(event_target_value(&event))
            >
                <Show when=move || organizations.get().is_empty()>
                    <option value="">{t("Organization")}</option>
                </Show>
                {move || {
                    organizations
                        .get()
                        .into_iter()
                        .map(|row| {
                            let id = row.id.clone();
                            view! {
                                <option value=row.id selected=move || selected.get() == id>
                                    {row.name}
                                </option>
                            }
                        })
                        .collect_view()
                }}
            </select>
        </label>
    }
}

#[component]
pub(super) fn OrganizationToolbar(state: OrganizationAdmin) -> impl IntoView {
    view! {
        <section class="admin-directory-scope organization-admin-scope">
            <OrganizationSelector
                id="selected-org"
                organizations=state.rows
                selected=state.selected
            />
            <div class="admin-directory-scope-badges">
                {move || {
                    state
                        .current
                        .get()
                        .map(|row| {
                            view! {
                                <span class="badge">{row.name}</span>
                                <span class="badge outline mono">{row.key}</span>
                                <span class="badge outline">{row.role}</span>
                            }
                        })
                }} <Show when=move || state.current.get().is_none()>
                    <span class="muted">{t("No organizations available.")}</span>
                </Show>
            </div>
            <div class="admin-directory-scope-actions">
                <button
                    type="button"
                    disabled=move || !state.can_manage.get()
                    on:click=move |_| {
                        state.clear_errors();
                        state.edit_open.set(true);
                    }
                >
                    <thaw::Icon icon=icondata_lu::LuSettings2 width="14px" height="14px" />
                    {t("Edit Organization")}
                </button>
                <button
                    type="button"
                    class="secondary"
                    disabled=move || !state.can_manage.get()
                    on:click=move |_| {
                        state.clear_errors();
                        state.member_open.set(true);
                    }
                >
                    <thaw::Icon icon=icondata_lu::LuUserPlus width="14px" height="14px" />
                    {t("Add Member")}
                </button>
                <button
                    type="button"
                    class="danger"
                    disabled=move || !state.can_manage.get() || state.deletion.busy.get()
                    on:click=move |_| {
                        state.clear_errors();
                        state.delete_open.set(true);
                    }
                >
                    {t("Delete Organization")}
                </button>
            </div>
        </section>
    }
}
