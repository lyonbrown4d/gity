mod asset_form;
mod forms;
mod listing;
mod model;
mod sidebar;
use crate::{api, i18n::t, ui::*};
use leptos::prelude::*;
use serde_json::Value;

#[derive(Clone, Copy)]
struct Context {
    path: StoredValue<String>,
    rows: Query,
    tags: Query,
    project: Query,
    permissions: Query,
    selected: RwSignal<String>,
    pending: RwSignal<String>,
    composer: RwSignal<bool>,
    link_composer: RwSignal<bool>,
    editing: RwSignal<Value>,
    form_key: RwSignal<u64>,
}
impl Context {
    fn can_admin(self) -> bool {
        allowed(
            &self.permissions.data.get(),
            "project.repository.admin",
            "repository_admin",
            40,
        )
    }
    fn refresh(self) {
        self.tags.refresh.update(|n| *n += 1);
    }
    fn active(self) -> Value {
        model::active(&self.rows.data.get(), &self.selected.get())
    }
    fn edit(self, value: Value) {
        self.editing.set(value);
        self.form_key.update(|n| *n += 1);
        self.composer.set(true);
    }
}

#[component]
pub(super) fn Releases(project_path: String, permissions: Query) -> impl IntoView {
    let context = Context {
        path: StoredValue::new(project_path.clone()),
        rows: query(format!("{project_path}/releases")),
        tags: query(format!("{project_path}/repository/tags")),
        project: query(project_path),
        permissions,
        selected: RwSignal::new(String::new()),
        pending: RwSignal::new(String::new()),
        composer: RwSignal::new(false),
        link_composer: RwSignal::new(false),
        editing: RwSignal::new(Value::Null),
        form_key: RwSignal::new(0),
    };
    Effect::new(move |previous: Option<u64>| {
        let refresh = context.tags.refresh.get();
        if previous.is_some() {
            context.rows.refresh.update(|n| *n += 1);
        }
        refresh
    });
    Effect::new(move |_| {
        let rows = api::list(&context.rows.data.get());
        let pending = context.pending.get();
        if !pending.is_empty() {
            if rows
                .iter()
                .any(|row| api::text(&row["release"], "id") == pending)
            {
                context.selected.set(pending);
                context.pending.set(String::new());
            }
            return;
        }
        let selected = context.selected.get();
        if !context.rows.loading.get()
            && !rows
                .iter()
                .any(|row| api::text(&row["release"], "id") == selected)
        {
            let next = rows
                .first()
                .map(|row| api::text(&row["release"], "id"))
                .unwrap_or_default();
            if next != selected {
                context.selected.set(next);
            }
        }
    });
    view! {
        <section class="releases-panel account-card card-enter">
            <header class="releases-header">
                <div>
                    <h2>{t("Releases")}</h2>
                    <p>
                        {t(
                            "Manage repository tags, release notes, and external release asset links.",
                        )}
                    </p>
                </div>
                <span class="releases-badge" class:releases-secondary=move || context.can_admin()>
                    {move || t(
                        if context.can_admin() { "Admin actions enabled" } else { "Read only" },
                    )}
                </span>
            </header>
            <div class="releases-body">
                <div class="releases-stats">
                    <Stat
                        label="Releases"
                        value=Signal::derive(move || api::list(&context.rows.data.get()).len())
                    />
                    <Stat
                        label="Tags"
                        value=Signal::derive(move || api::list(&context.tags.data.get()).len())
                    />
                    <Stat
                        label="Asset links"
                        value=Signal::derive(move || model::link_count(&context.rows.data.get()))
                    />
                </div>
                <div class="releases-between">
                    <div class="releases-actions">
                        <button
                            type="button"
                            class:primary=move || !context.composer.get()
                            disabled=move || !context.can_admin()
                            on:click=move |_| {
                                if context.composer.get() {
                                    context.composer.set(false);
                                } else {
                                    context.edit(Value::Null);
                                }
                            }
                        >
                            <thaw::Icon icon=icondata_lu::LuPlus width="16px" height="16px" />
                            {move || t(
                                if context.composer.get() {
                                    "Hide release form"
                                } else {
                                    "New release"
                                },
                            )}
                        </button>
                        <button
                            type="button"
                            disabled=move || !context.can_admin() || context.active().is_null()
                            on:click=move |_| context.link_composer.update(|open| *open = !*open)
                        >
                            <thaw::Icon icon=icondata_lu::LuLink2 width="16px" height="16px" />
                            {move || t(
                                if context.link_composer.get() {
                                    "Hide link form"
                                } else {
                                    "Add asset link"
                                },
                            )}
                        </button>
                    </div>
                    <button
                        type="button"
                        class="releases-ghost"
                        on:click=move |_| context.refresh()
                    >
                        <thaw::Icon icon=icondata_lu::LuRefreshCw width="14px" height="14px" />
                        {t("Reload")}
                    </button>
                </div>
                <Show when=move || !context.permissions.loading.get() && !context.can_admin()>
                    <aside class="releases-box">
                        <strong>{t("Release management is read-only")}</strong>
                        <p>
                            {t(
                                "Your current project role can inspect releases, but cannot change them.",
                            )}
                        </p>
                    </aside>
                </Show>
                <Show when=move || {
                    context.composer.get()
                }>
                    {move || {
                        context.form_key.get();
                        view! { <forms::ReleaseForm context=context /> }
                    }}
                </Show>
                <Show when=move || context.link_composer.get() && !context.active().is_null()>
                    <asset_form::AssetForm context=context />
                </Show>
                <div class="releases-workspace">
                    <listing::Listing context=context />
                    <sidebar::Sidebar context=context />
                </div>
            </div>
        </section>
    }
}
#[component]
fn Stat(label: &'static str, value: Signal<usize>) -> impl IntoView {
    view! {
        <div class="releases-stat">
            <small>{t(label)}</small>
            <strong>{move || value.get()}</strong>
        </div>
    }
}
#[component]
fn Empty(title: String, description: String) -> impl IntoView {
    view! {
        <div class="releases-empty">
            <thaw::Icon icon=icondata_lu::LuTag width="20px" height="20px" />
            <div>
                <strong>{title}</strong>
                <p>{description}</p>
            </div>
        </div>
    }
}
#[component]
fn Remove(
    label: String,
    title: String,
    description: String,
    path: String,
    refresh: RwSignal<u64>,
    can_admin: Signal<bool>,
    #[prop(default = true)] confirm: bool,
) -> impl IntoView {
    view! {
        <span class="releases-remove" on:click=move |e| e.stop_propagation()>
            <Action
                label=label
                path=path
                method="DELETE".into()
                refresh=refresh
                confirm=confirm
                disabled=Signal::derive(move || !can_admin.get())
                confirmation_title=title
                confirmation_description=description
            />
            <thaw::Icon icon=icondata_lu::LuTrash2 width="16px" height="16px" />
        </span>
    }
}
