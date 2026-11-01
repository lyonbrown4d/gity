use super::actions::CopyCloneUrl;
use crate::{app::admin_orgs::model::ProjectRow, i18n::t};
use leptos::prelude::*;

#[component]
pub(super) fn ProjectTable(
    rows: Memo<Vec<ProjectRow>>,
    on_delete: Callback<ProjectRow>,
    busy: Signal<bool>,
    error: Signal<String>,
) -> impl IntoView {
    view! {
        <div class="table-scroll">
            <table class="admin-directory-table admin-projects-table">
                <thead>
                    <tr>
                        <th>{t("Project")}</th>
                        <th>{t("Clone URL")}</th>
                        <th>{t("Visibility")}</th>
                        <th>{t("Actions")}</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        rows
                            .get()
                            .into_iter()
                            .map(|project| {
                                let deletion = project.clone();
                                let label = format!("{} {}", t("Delete"), project.name);
                                view! {
                                    <tr>
                                        <td>
                                            <div class="admin-directory-identity project-identity">
                                                <i class:active=project.status == "active"></i>
                                                <div>
                                                    <p>{project.name.clone()}</p>
                                                    <small class="mono">
                                                        {project.key}" / "{project.default_branch}
                                                    </small>
                                                    {(!project.description.is_empty())
                                                        .then(|| {
                                                            view! {
                                                                <small class="project-description">
                                                                    {project.description}
                                                                </small>
                                                            }
                                                        })}
                                                </div>
                                            </div>
                                        </td>
                                        <td class="project-clone-address">
                                            <p title=project
                                                .clone_http_url
                                                .clone()>{project.clone_http_url.clone()}</p>
                                            <small>{project.uuid}</small>
                                        </td>
                                        <td>
                                            <span class="badge outline">{project.visibility}</span>
                                        </td>
                                        <td>
                                            <div class="admin-project-actions">
                                                <CopyCloneUrl
                                                    value=project.clone_http_url
                                                    name=project.name
                                                />
                                                <button
                                                    type="button"
                                                    class="project-delete"
                                                    aria-label=label
                                                    title=t("Delete")
                                                    disabled=move || busy.get()
                                                    on:click=move |_| on_delete.run(deletion.clone())
                                                >
                                                    <thaw::Icon
                                                        icon=icondata_lu::LuTrash2
                                                        width="14px"
                                                        height="14px"
                                                    />
                                                </button>
                                            </div>
                                        </td>
                                    </tr>
                                }
                            })
                            .collect_view()
                    }} <Show when=move || rows.get().is_empty() && error.get().is_empty()>
                        <tr>
                            <td colspan="4" class="admin-directory-empty">
                                <thaw::Icon
                                    icon=icondata_lu::LuCircleDotDashed
                                    width="20px"
                                    height="20px"
                                />
                                {t("No projects in this organization.")}
                            </td>
                        </tr>
                    </Show>
                </tbody>
            </table>
        </div>
    }
}
