mod checks;
mod collaboration;
mod comments;
mod composer;
mod detail;
mod listing;
mod model;
mod rule_form;
mod rules;
use crate::{api, i18n::t, ui::*};
use leptos::prelude::*;
use serde_json::Value;

#[derive(Clone, Copy)]
struct Context {
    path: StoredValue<String>,
    rows: Query,
    users: Query,
    branches: Query,
    project: Query,
    permissions: Query,
    selected: RwSignal<String>,
    comment_draft: RwSignal<String>,
    pending: RwSignal<String>,
    composer: RwSignal<bool>,
    rules_changed: RwSignal<u64>,
    gates: RwSignal<Value>,
    gates_loading: RwSignal<bool>,
    gates_error: RwSignal<String>,
    state: RwSignal<String>,
    search: RwSignal<String>,
    source: RwSignal<String>,
    target: RwSignal<String>,
}
impl Context {
    fn select(self, iid: String) {
        if self.selected.get_untracked() != iid {
            self.comment_draft.set(String::new());
            self.selected.set(iid);
        }
    }

    fn can(self, action: &str, level: u64) -> bool {
        allowed(
            &self.permissions.data.get(),
            &format!("project.merge_requests.{action}"),
            "",
            level,
        )
    }
    fn reset_filters(self) {
        self.state.set("opened".into());
        self.search.set(String::new());
        self.source.set(String::new());
        self.target.set(String::new());
    }
    fn filtered(self) -> Vec<Value> {
        model::filter(
            &api::list(&self.rows.data.get()),
            &self.state.get(),
            &self.search.get(),
            &self.source.get(),
            &self.target.get(),
        )
    }
}

#[component]
pub(super) fn MergeRequests(project_path: String, permissions: Query) -> impl IntoView {
    let context = Context {
        path: StoredValue::new(project_path.clone()),
        rows: query(format!("{project_path}/merge-requests")),
        users: query_all_pages("/users".into()),
        branches: query(format!("{project_path}/repository/branches")),
        project: query(project_path),
        permissions,
        selected: RwSignal::new(String::new()),
        comment_draft: RwSignal::new(String::new()),
        pending: RwSignal::new(String::new()),
        composer: RwSignal::new(false),
        rules_changed: RwSignal::new(0),
        gates: RwSignal::new(Value::Null),
        gates_loading: RwSignal::new(false),
        gates_error: RwSignal::new(String::new()),
        state: RwSignal::new("opened".into()),
        search: RwSignal::new(String::new()),
        source: RwSignal::new(String::new()),
        target: RwSignal::new(String::new()),
    };
    Effect::new(move |_| {
        let rows = api::list(&context.rows.data.get());
        let pending = context.pending.get();
        if !pending.is_empty() {
            if rows.iter().any(|row| api::text(row, "iid") == pending) {
                context.select(pending);
                context.pending.set(String::new());
            }
            return;
        }
        let selected = context.selected.get();
        if !context.rows.loading.get() && !rows.iter().any(|row| api::text(row, "iid") == selected)
        {
            let next = rows
                .first()
                .map(|row| api::text(row, "iid"))
                .unwrap_or_default();
            if next != selected {
                context.select(next);
            }
        }
    });
    view! {
        <section class="mr-panel account-card card-enter">
            <header class="mr-header">
                <h2>{t("Merge Requests")}</h2>
                <p>{t("Review branch changes and merge them into protected targets.")}</p>
            </header>
            <div class="mr-body">
                <Status q=context.project />
                <Status q=context.branches />
                <Show when=move || !context.users.error.get().is_empty()>
                    <p class="error" role="alert">
                        {move || context.users.error.get()}
                    </p>
                </Show>
                <div class="mr-stats">
                    <Stat
                        label="Open"
                        value=Signal::derive(move || model::count(
                            &context.rows.data.get(),
                            "opened",
                        ))
                    />
                    <Stat
                        label="Merged"
                        value=Signal::derive(move || model::count(
                            &context.rows.data.get(),
                            "merged",
                        ))
                    />
                    <Stat
                        label="Closed"
                        value=Signal::derive(move || model::count(
                            &context.rows.data.get(),
                            "closed",
                        ))
                    />
                </div>
                <rules::Rules context=context />
                <div class="mr-workspace">
                    <listing::Listing context=context />
                    <div class="mr-detail mr-box">
                        {move || {
                            let active = api::list(&context.rows.data.get())
                                .into_iter()
                                .find(|row| api::text(row, "iid") == context.selected.get());
                            match active {
                                Some(mr) => {
                                    view! { <detail::Detail context=context mr=mr /> }.into_any()
                                }
                                None => {
                                    view! {
                                        <p class="mr-muted">
                                            {t("Select a merge request to inspect the diff.")}
                                        </p>
                                    }
                                        .into_any()
                                }
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
        <div class="mr-stat">
            <small>{t(label)}</small>
            <strong>{move || value.get()}</strong>
        </div>
    }
}
#[component]
fn Badge(text: String, #[prop(default = false)] primary: bool) -> impl IntoView {
    view! {
        <span class="mr-badge" class:mr-primary=primary>
            {text}
        </span>
    }
}
#[component]
fn Reload(q: Query) -> impl IntoView {
    view! {
        <button class="mr-ghost" type="button" on:click=move |_| q.refresh.update(|v| *v += 1)>
            <thaw::Icon icon=icondata_lu::LuRefreshCw width="14px" height="14px" />
            {t("Reload")}
        </button>
    }
}
#[component]
fn MutationError(mutation: Mutation) -> impl IntoView {
    view! {
        <Show when=move || !mutation.error.get().is_empty()>
            <p class="error" role="alert">
                {move || mutation.error.get()}
            </p>
        </Show>
    }
}
