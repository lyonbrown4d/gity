use crate::{api, ui::*};
use leptos::prelude::*;
#[component]
pub(super) fn Intelligence(project_path: String, branch: RwSignal<String>) -> impl IntoView {
    let q = query_fn(
        move || {
            format!(
                "{project_path}/repository/languages?branch_name={}",
                api::enc(&branch.get())
            )
        },
        true,
    );
    view! {
        <aside class="source-intelligence">
            <div class="source-strip">
                <strong>{crate::i18n::t("◇ Repository pulse")}</strong>
                <span class="badge outline mono">{move || branch.get()}</span>
            </div>
            <section>
                <div class="source-strip">
                    <h2>{crate::i18n::t("▥ Languages")}</h2>
                    <button
                        title="Refresh"
                        aria-label="Refresh languages"
                        disabled=move || q.loading.get()
                        on:click=move |_| q.refresh.update(|v| *v += 1)
                    >
                        {crate::i18n::t("↻")}
                    </button>
                </div>
                <Status q=q />
                {move || {
                    q.data
                        .get()
                        .get("languages")
                        .map(api::list)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|item| {
                            let percentage = item["percentage"].as_f64().unwrap_or(0.);
                            view! {
                                <div class="language-row">
                                    <div>
                                        <strong>{api::text(&item, "language")}</strong>
                                        <small>{format!("{percentage:.1}%")}</small>
                                    </div>
                                    <div class="language-track">
                                        <span style:width=format!("{percentage}%")></span>
                                    </div>
                                </div>
                            }
                        })
                        .collect_view()
                }}
                <Show when=move || {
                    !q.loading.get()
                        && q
                            .data
                            .get()
                            .get("languages")
                            .map(api::list)
                            .unwrap_or_default()
                            .is_empty()
                }>
                    <p class="muted">{crate::i18n::t("No language statistics yet.")}</p>
                </Show>
            </section>
            <div class="language-metadata">
                <div>
                    <small>{crate::i18n::t("Total size")}</small>
                    <strong>{move || api::text(&q.data.get(), "total_bytes")}" B"</strong>
                </div>
                <div>
                    <small>{crate::i18n::t("Last analyzed")}</small>
                    <strong>{move || api::text(&q.data.get(), "analyzed_at")}</strong>
                </div>
            </div>
        </aside>
    }
}
