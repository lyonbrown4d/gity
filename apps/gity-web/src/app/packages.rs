mod detail;
mod forms;
mod model;
use crate::{api, i18n::t, ui::*};
use leptos::prelude::*;
use serde_json::Value;

#[component]
pub(super) fn Packages(project_path: String, permissions: Query) -> impl IntoView {
    let packages = query(format!("{project_path}/packages"));
    let selected = RwSignal::new(String::new());
    let composer = RwSignal::new(false);
    let can_write = Signal::derive(move || {
        allowed(
            &permissions.data.get(),
            "project.packages.write",
            "package_write",
            30,
        )
    });
    let current = Memo::new(move |_| {
        api::list(&packages.data.get())
            .into_iter()
            .find(|row| api::text(row, "id") == selected.get())
            .unwrap_or(Value::Null)
    });
    Effect::new(move |_| {
        let rows = api::list(&packages.data.get());
        if !rows
            .iter()
            .any(|row| api::text(row, "id") == selected.get())
        {
            let next = rows
                .first()
                .map(|row| api::text(row, "id"))
                .unwrap_or_default();
            if next != selected.get_untracked() {
                selected.set(next);
            }
        }
    });
    let detail_path = project_path.clone();
    let detail = query_fn(
        move || {
            let id = selected.get();
            if id.is_empty() {
                String::new()
            } else {
                format!("{detail_path}/packages/{}", api::enc(&id))
            }
        },
        true,
    );
    let refresh = move |_| {
        packages.refresh.update(|v| *v += 1);
        detail.refresh.update(|v| *v += 1);
    };
    let saved = Callback::new(move |_: ()| {
        composer.set(false);
        selected.set(String::new());
        packages.refresh.update(|v| *v += 1);
        detail.refresh.update(|v| *v += 1);
    });
    let upload_path = format!("{project_path}/packages/files");
    view! {
        <section class="packages-panel account-card card-enter">
            <header class="packages-header">
                <div>
                    <h2>{t("Package Registry")}</h2>
                    <p>{t("Publish generic project artifacts and inspect package versions.")}</p>
                </div>
                <span class="badge" class:outline=move || !can_write.get()>
                    {move || t(if can_write.get() { "Upload enabled" } else { "Read only" })}
                </span>
            </header>
            <div class="packages-body">
                <div class="packages-stats">
                    <Stat
                        label="Packages"
                        value=Signal::derive(move || api::list(&packages.data.get()).len())
                    />
                    <Stat
                        label="Versions"
                        value=Signal::derive(move || model::versions(&detail.data.get()).len())
                    />
                    <Stat
                        label="Files"
                        value=Signal::derive(move || model::file_count(&detail.data.get()))
                    />
                </div>
                <div class="packages-toolbar">
                    <button
                        type="button"
                        class="primary"
                        disabled=move || !can_write.get()
                        on:click=move |_| composer.update(|v| *v = !*v)
                    >
                        <thaw::Icon icon=icondata_lu::LuPlus width="16px" height="16px" />
                        {move || t(
                            if composer.get() { "Hide upload form" } else { "Upload package file" },
                        )}
                    </button>
                    <button type="button" class="packages-ghost" on:click=refresh>
                        <thaw::Icon icon=icondata_lu::LuRefreshCw width="14px" height="14px" />
                        {t("Reload")}
                    </button>
                </div>
                <Show when=move || !can_write.get()>
                    <div class="packages-notice">
                        <strong>{t("Package registry is read-only")}</strong>
                        <p>
                            {t(
                                "Your current project role can inspect packages, but cannot upload them.",
                            )}
                        </p>
                    </div>
                </Show>
                <Show when=move || composer.get()>
                    <forms::UploadForm
                        path=upload_path.clone()
                        can_write=can_write
                        on_saved=saved
                    />
                </Show>
                <Status q=packages />
                <div class="packages-workspace">
                    <nav class="packages-list" aria-label=t("Packages")>
                        <Show when=move || {
                            !packages.loading.get() && api::list(&packages.data.get()).is_empty()
                        }>
                            <Empty
                                title="No packages published yet."
                                description=Signal::derive(move || {
                                    if can_write.get() {
                                        "Upload the first package file to create a version."
                                    } else {
                                        "Packages will appear here after maintainers publish artifacts."
                                    }
                                        .to_string()
                                })
                            />
                        </Show>
                        {move || {
                            api::list(&packages.data.get())
                                .into_iter()
                                .map(|row| {
                                    let id = api::text(&row, "id");
                                    let key = id.clone();
                                    view! {
                                        <button
                                            type="button"
                                            class="packages-item"
                                            class:selected=move || selected.get() == key
                                            on:click=move |_| selected.set(id.clone())
                                        >
                                            <div>
                                                <thaw::Icon
                                                    icon=icondata_lu::LuPackage
                                                    width="16px"
                                                    height="16px"
                                                />
                                                <div>
                                                    <strong>{api::text(&row, "name")}</strong>
                                                    <small>{api::text(&row, "type")}</small>
                                                </div>
                                            </div>
                                            <small>
                                                {relative_time(&api::text(&row, "updated_at"))}
                                            </small>
                                        </button>
                                    }
                                })
                                .collect_view()
                        }}
                    </nav>
                    <div class="packages-document">
                        {move || {
                            if current.get().is_object() {
                                view! {
                                    <detail::PackageDetail
                                        package=current.get()
                                        query=detail
                                        project_path=project_path.clone()
                                    />
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <Empty
                                        title="Select or upload a package."
                                        description=Signal::derive(move || {
                                            if can_write.get() {
                                                "Upload a package file or select an existing package to inspect versions."
                                            } else {
                                                "Select an existing package to inspect versions and files."
                                            }
                                                .to_string()
                                        })
                                    />
                                }
                                    .into_any()
                            }
                        }}
                    </div>
                </div>
            </div>
        </section>
    }
}
#[component]
fn Stat(label: &'static str, value: Signal<usize>) -> impl IntoView {
    view! {
        <div class="packages-stat">
            <small>{t(label)}</small>
            <strong>{move || value.get()}</strong>
        </div>
    }
}
#[component]
fn Empty(title: &'static str, description: Signal<String>) -> impl IntoView {
    view! {
        <div class="packages-empty">
            <thaw::Icon icon=icondata_lu::LuPackage width="20px" height="20px" />
            <strong>{t(title)}</strong>
            <small>{move || t(&description.get())}</small>
        </div>
    }
}
