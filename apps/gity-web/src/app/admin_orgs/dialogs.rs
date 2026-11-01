use super::{
    controller::OrganizationAdmin,
    forms::{DialogClose, MemberForm, OrganizationForm},
};
use crate::{api, i18n::t};
use leptos::prelude::*;
use serde_json::json;

#[component]
pub(super) fn OrganizationDialogs(state: OrganizationAdmin) -> impl IntoView {
    view! {
        <thaw::Dialog open=state.create_open>
            <thaw::DialogSurface class="admin-directory-dialog">
                <DialogClose open=state.create_open />
                <thaw::DialogBody>
                    <thaw::DialogTitle>{t("Create Organization")}</thaw::DialogTitle>
                    <thaw::DialogContent>
                        <Show when=move || state.create_open.get()>
                            <OrganizationForm
                                initial=None
                                mutation=state.creation
                                on_cancel=Callback::new(move |_| state.create_open.set(false))
                                on_saved=Callback::new(move |value: serde_json::Value| {
                                    state.create_open.set(false);
                                    state.selected.set(api::text(&value, "id"));
                                })
                            />
                        </Show>
                    </thaw::DialogContent>
                </thaw::DialogBody>
            </thaw::DialogSurface>
        </thaw::Dialog>
        <thaw::Dialog open=state.edit_open>
            <thaw::DialogSurface class="admin-directory-dialog">
                <DialogClose open=state.edit_open />
                <thaw::DialogBody>
                    <thaw::DialogTitle>{t("Edit Organization")}</thaw::DialogTitle>
                    <thaw::DialogContent>
                        <Show when=move || state.edit_open.get() && state.current.get().is_some()>
                            <OrganizationForm
                                initial=state.current.get()
                                mutation=state.update
                                on_cancel=Callback::new(move |_| state.edit_open.set(false))
                                on_saved=Callback::new(move |_| state.edit_open.set(false))
                            />
                        </Show>
                    </thaw::DialogContent>
                </thaw::DialogBody>
            </thaw::DialogSurface>
        </thaw::Dialog>
        <thaw::Dialog open=state.member_open>
            <thaw::DialogSurface class="admin-directory-dialog">
                <DialogClose open=state.member_open />
                <thaw::DialogBody>
                    <thaw::DialogTitle>{t("Add Organization Member")}</thaw::DialogTitle>
                    <thaw::DialogContent>
                        <Show when=move || state.member_open.get()>
                            <MemberForm
                                organization_id=state.selected.get()
                                mutation=state.member
                                on_cancel=Callback::new(move |_| state.member_open.set(false))
                                on_saved=Callback::new(move |_| state.member_open.set(false))
                            />
                        </Show>
                    </thaw::DialogContent>
                </thaw::DialogBody>
            </thaw::DialogSurface>
        </thaw::Dialog>
        <thaw::Dialog open=state.delete_open>
            <thaw::DialogSurface class="admin-directory-dialog">
                <DialogClose open=state.delete_open />
                <thaw::DialogBody>
                    <thaw::DialogTitle>
                        {move || gity_i18n::translate(
                            "Delete organization \"{name}\"?",
                            &crate::i18n::locale(),
                            &[
                                (
                                    "name",
                                    &state.current.get().map(|row| row.name).unwrap_or_default(),
                                ),
                            ],
                        )}
                    </thaw::DialogTitle>
                    <thaw::DialogContent>
                        <p>{t("This action cannot be undone.")}</p>
                        <Show when=move || !state.deletion.error.get().is_empty()>
                            <p class="error" role="alert">
                                {move || state.deletion.error.get()}
                            </p>
                        </Show>
                        <footer class="admin-directory-confirm-actions">
                            <button type="button" on:click=move |_| state.delete_open.set(false)>
                                {t("Cancel")}
                            </button>
                            <button
                                type="button"
                                class="danger"
                                disabled=move || {
                                    state.deletion.busy.get() || !state.can_manage.get()
                                }
                                on:click=move |_| {
                                    state
                                        .deletion
                                        .run(
                                            "DELETE",
                                            format!(
                                                "/orgs/{}",
                                                api::enc(&state.selected.get_untracked()),
                                            ),
                                            json!({}),
                                            Some(Callback::new(move |_| state.delete_open.set(false))),
                                        )
                                }
                            >
                                {move || t(
                                    if state.deletion.busy.get() { "Deleting..." } else { "Delete" },
                                )}
                            </button>
                        </footer>
                    </thaw::DialogContent>
                </thaw::DialogBody>
            </thaw::DialogSurface>
        </thaw::Dialog>
    }
}
