mod model;
use crate::{api, i18n::t, ui::*};
use leptos::prelude::*;
use model::{Event, Stats};

#[component]
pub(super) fn Audit(project_path: String) -> impl IntoView {
    let query = query(format!("{project_path}/audit-events?limit=100"));
    let events = Memo::new(move |_| {
        api::list(&query.data.get())
            .iter()
            .map(Event::from)
            .collect::<Vec<_>>()
    });
    let stats = Memo::new(move |_| Stats::from_events(&events.get()));
    let initial_loading = Signal::derive(move || query.loading.get() && query.data.get().is_null());
    view! {
        <section class="audit-panel account-card card-enter">
            <header class="audit-header">
                <div>
                    <h2>{t("Audit")}</h2>
                    <p>
                        {t(
                            "Review project audit events emitted by repository, CI, and collaboration workflows.",
                        )}
                    </p>
                </div>
                <div class="audit-actions">
                    <span class="audit-badge" class:secondary=move || !initial_loading.get()>
                        {move || t(if initial_loading.get() { "Loading" } else { "Latest 100" })}
                    </span>
                    <button
                        type="button"
                        class="audit-reload"
                        on:click=move |_| query.refresh.update(|n| *n += 1)
                    >
                        <thaw::Icon icon=icondata_lu::LuRefreshCw width="14px" height="14px" />
                        {t("Reload")}
                    </button>
                </div>
            </header>
            <div class="audit-body">
                <div class="audit-stats">
                    <Stat label="Events" value=Signal::derive(move || stats.get().events) />
                    <Stat label="Actors" value=Signal::derive(move || stats.get().actors) />
                    <Stat
                        label="Target types"
                        value=Signal::derive(move || stats.get().target_types)
                    />
                </div>
                <Show when=move || !query.error.get().is_empty()>
                    <Status q=query />
                </Show>
                <div class="audit-table">
                    <table>
                        <thead>
                            <tr>
                                <th>{t("Event")}</th>
                                <th>{t("Actor")}</th>
                                <th>{t("Target")}</th>
                                <th>{t("Summary")}</th>
                                <th>{t("Time")}</th>
                            </tr>
                        </thead>
                        <tbody>
                            <Show when=move || initial_loading.get()>
                                <tr>
                                    <td colspan="5" class="audit-muted">
                                        {t("Loading audit events...")}
                                    </td>
                                </tr>
                            </Show>
                            <Show when=move || {
                                !initial_loading.get() && query.error.get().is_empty()
                                    && events.get().is_empty()
                            }>
                                <tr>
                                    <td colspan="5">
                                        <div class="audit-empty">
                                            <strong>{t("No audit events found.")}</strong>
                                            <p>
                                                {t(
                                                    "Repository, CI, and collaboration activity will appear here after events are emitted.",
                                                )}
                                            </p>
                                        </div>
                                    </td>
                                </tr>
                            </Show>
                            {move || {
                                events
                                    .get()
                                    .into_iter()
                                    .map(|event| view! { <EventRow event=event /> })
                                    .collect_view()
                            }}
                        </tbody>
                    </table>
                </div>
            </div>
        </section>
    }
}
#[component]
fn Stat(label: &'static str, value: Signal<usize>) -> impl IntoView {
    view! {
        <div class="audit-stat">
            <small>{t(label)}</small>
            <strong>{move || value.get()}</strong>
        </div>
    }
}
#[component]
fn EventRow(event: Event) -> impl IntoView {
    let label = if event.name.is_empty() {
        event.action.clone()
    } else {
        event.name
    };
    let summary = if event.summary.is_empty() {
        t("N/A")
    } else {
        event.summary
    };
    view! {
        <tr>
            <td>
                <div class="audit-event">
                    <thaw::Icon icon=icondata_lu::LuScrollText width="16px" height="16px" />
                    <strong>{label}</strong>
                    {if event.action.is_empty() {
                        None
                    } else {
                        Some(view! { <span class="audit-badge">{event.action}</span> })
                    }}
                </div>
            </td>
            <td class="audit-actor">{format!("#{}", event.actor)}</td>
            <td class="audit-meta">
                {if event.target_type.is_empty() { t("N/A") } else { event.target_type }}
                {if event.target_id.is_empty() {
                    String::new()
                } else {
                    format!(" #{}", event.target_id)
                }}
            </td>
            <td class="audit-summary" title=summary.clone()>
                {summary.clone()}
            </td>
            <td class="audit-meta">
                {if event.created_at.is_empty() {
                    t("N/A")
                } else {
                    relative_time(&event.created_at)
                }}
            </td>
        </tr>
    }
}
