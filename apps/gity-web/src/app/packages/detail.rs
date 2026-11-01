use super::model;
use crate::{api, i18n::t, ui::*};
use leptos::prelude::*;
use serde_json::Value;

#[component]
pub(super) fn PackageDetail(package: Value, query: Query, project_path: String) -> impl IntoView {
    view! {
        <div class="packages-detail">
            <header>
                <div>
                    <div>
                        <h3>{api::text(&package, "name")}</h3>
                        <span class="badge">{api::text(&package, "type")}</span>
                    </div>
                    <small>
                        {format!(
                            "#{} · {}",
                            api::text(&package, "id"),
                            relative_time(&api::text(&package, "updated_at")),
                        )}
                    </small>
                </div>
                <button
                    type="button"
                    class="packages-ghost"
                    on:click=move |_| query.refresh.update(|v| *v += 1)
                >
                    <thaw::Icon icon=icondata_lu::LuRefreshCw width="14px" height="14px" />
                    {t("Reload detail")}
                </button>
            </header>
            <Status q=query />
            <Show when=move || {
                !query.loading.get() && model::versions(&query.data.get()).is_empty()
            }>
                <div class="packages-notice">
                    <strong>{t("No package versions found.")}</strong>
                    <p>
                        {t("Upload a package file to create the first version for this package.")}
                    </p>
                </div>
            </Show>
            {move || {
                model::versions(&query.data.get())
                    .into_iter()
                    .map(|row| {
                        let version = &row["version"];
                        let project_path = project_path.clone();
                        view! {
                            <section class="packages-version">
                                <header>
                                    <div>
                                        <strong>{api::text(version, "version")}</strong>
                                        <small>
                                            {relative_time(&api::text(version, "updated_at"))}
                                        </small>
                                    </div>
                                    <span class="badge outline">
                                        {api::text(version, "status")}
                                    </span>
                                </header>
                                <div class="packages-table-scroll">
                                    <table>
                                        <thead>
                                            <tr>
                                                <th>{t("File")}</th>
                                                <th>{t("Path")}</th>
                                                <th>{t("Size")}</th>
                                                <th>{t("Actions")}</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {api::list(&row["files"])
                                                .into_iter()
                                                .map(|file| {
                                                    view! {
                                                        <tr>
                                                            <td>{api::text(&file, "file_name")}</td>
                                                            <td title=api::text(
                                                                &file,
                                                                "file_path",
                                                            )>{api::text(&file, "file_path")}</td>
                                                            <td>
                                                                {model::bytes(
                                                                    file["byte_size"].as_u64().unwrap_or_default(),
                                                                )}
                                                            </td>
                                                            <td>
                                                                <crate::files::Download
                                                                    path=format!(
                                                                        "{project_path}/packages/files/{}",
                                                                        api::enc(&api::text(&file, "id")),
                                                                    )
                                                                    filename=api::text(&file, "file_name")
                                                                />
                                                            </td>
                                                        </tr>
                                                    }
                                                })
                                                .collect_view()}
                                            {if api::list(&row["files"]).is_empty() {
                                                Some(
                                                    view! {
                                                        <tr>
                                                            <td colspan="4">{t("No files in this version.")}</td>
                                                        </tr>
                                                    },
                                                )
                                            } else {
                                                None
                                            }}
                                        </tbody>
                                    </table>
                                </div>
                            </section>
                        }
                    })
                    .collect_view()
            }}
        </div>
    }
}
