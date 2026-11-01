use crate::i18n::t;
use leptos::prelude::*;

pub(super) struct ConsoleMetric {
    pub label: &'static str,
    pub value: Signal<String>,
    pub detail: String,
    pub active: Signal<bool>,
}

#[component]
pub(super) fn ConsoleHeader(
    title: &'static str,
    description: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <header class="console-page-header">
            <div>
                <p>{t("Admin Side")}</p>
                <h1>{t(title)}</h1>
                <small>{t(description)}</small>
            </div>
            <div class="console-header-actions">{children()}</div>
        </header>
    }
}

#[component]
pub(super) fn ConsoleMetrics(items: Vec<ConsoleMetric>) -> impl IntoView {
    view! {
        <dl class="console-metrics">
            {items
                .into_iter()
                .map(|item| {
                    view! {
                        <div>
                            <dt>
                                <i class:active=move || item.active.get()></i>
                                {t(item.label)}
                            </dt>
                            <dd>
                                <strong>{move || item.value.get()}</strong>
                                <span>{item.detail}</span>
                            </dd>
                        </div>
                    }
                })
                .collect_view()}
        </dl>
    }
}

#[component]
pub(super) fn ConsolePanel(
    title: &'static str,
    description: &'static str,
    meta: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <section class="console-panel">
            <header>
                <div>
                    <h2>{t(title)}</h2>
                    <p>{t(description)}</p>
                </div>
                <small>{move || meta.get()}</small>
            </header>
            {children()}
        </section>
    }
}

#[component]
pub(super) fn ConsolePagination(
    page: RwSignal<u64>,
    page_size: RwSignal<u64>,
    total: Signal<usize>,
    id_prefix: &'static str,
) -> impl IntoView {
    let pages =
        Signal::derive(move || (total.get() as u64).div_ceil(page_size.get().max(1)).max(1));
    Effect::new(move |_| {
        let maximum = pages.get();
        if page.get() > maximum {
            page.set(maximum);
        }
    });
    view! {
        <footer class="console-pagination users-pagination">
            <p>{t("Page")}" "{move || page.get()}" / "{move || pages.get()}</p>
            <div>
                <label for=format!("{id_prefix}-page-size")>{t("Page size")}</label>
                <select
                    id=format!("{id_prefix}-page-size")
                    aria-label=t("Page size")
                    prop:value=move || page_size.get().to_string()
                    on:change=move |event| {
                        if let Ok(value) = event_target_value(&event).parse::<u64>()
                            && [10, 20, 50, 100].contains(&value)
                        {
                            page_size.set(value);
                            page.set(1);
                        }
                    }
                >
                    {[10, 20, 50, 100]
                        .map(|value| view! { <option value=value.to_string()>{value}</option> })
                        .collect_view()}
                </select>
                <button
                    type="button"
                    disabled=move || page.get() <= 1
                    on:click=move |_| page.update(|value| *value = value.saturating_sub(1).max(1))
                >
                    {t("Prev")}
                </button>
                <button
                    type="button"
                    disabled=move || { page.get() >= pages.get() }
                    on:click=move |_| {
                        let maximum = pages.get_untracked();
                        page.update(|value| *value = value.saturating_add(1).min(maximum));
                    }
                >
                    {t("Next")}
                </button>
            </div>
        </footer>
    }
}
