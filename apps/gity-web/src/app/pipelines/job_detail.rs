use super::*;

#[component]
pub(super) fn JobDetail(context: Context, job: Job) -> impl IntoView {
    let id = StoredValue::new(job.id.clone());
    let job = StoredValue::new(job);
    let trace = Memo::new(move |_| {
        context
            .trace
            .data
            .with(|value| model::trace(value, &id.get_value()))
    });
    view! {
        <div class="pipeline-job-detail pipeline-box pipeline-stack">
            <div class="pipeline-between">
                <div class="pipeline-stack">
                    <div class="pipeline-actions">
                        <thaw::Icon icon=icondata_lu::LuScrollText width="16px" height="16px" />
                        <strong>
                            {format!("{}: {}", t("Job detail"), job.get_value().title())}
                        </strong>
                        <Badge state=job.get_value().state />
                    </div>
                    <p class="pipeline-muted">
                        {move || {
                            let (_, duration, exit) = trace.get().unwrap_or_default();
                            format!(
                                "{}: {} · {}: {}ms · {}: {}",
                                t("Job ID"),
                                id.get_value(),
                                t("Duration"),
                                duration,
                                t("Exit code"),
                                exit,
                            )
                        }}
                    </p>
                </div>
                <button
                    type="button"
                    class="pipeline-ghost"
                    on:click=move |_| {
                        context.trace.refresh.update(|n| *n += 1);
                        context.artifacts.refresh.update(|n| *n += 1);
                    }
                >
                    <thaw::Icon icon=icondata_lu::LuRefreshCw width="14px" height="14px" />
                    {t("Reload job")}
                </button>
            </div>
            <div class="pipeline-output-grid">
                <div class="pipeline-stack pipeline-trace">
                    <h4>{t("Trace")}</h4>
                    <QueryError q=context.trace />
                    <Show when=move || context.trace.loading.get()>
                        <div class="loading-placeholder" style="height:192px" />
                    </Show>
                    {move || {
                        let output = trace
                            .get()
                            .map(|trace| trace.0)
                            .filter(|text| !text.is_empty())
                            .unwrap_or_else(|| job.get_value().error);
                        if context.trace.loading.get() {
                            return ().into_any();
                        }
                        if output.is_empty() {
                            view! {
                                <Notice
                                    title="No trace captured yet"
                                    message=messages::empty_trace(job.get_value().state)
                                />
                            }
                                .into_any()
                        } else {
                            view! { <pre class="pipeline-trace-output">{output}</pre> }.into_any()
                        }
                    }}
                </div>
                <div class="pipeline-stack pipeline-artifacts">
                    <h4>{t("Artifacts")}</h4>
                    <QueryError q=context.artifacts />
                    <Show when=move || context.artifacts.loading.get()>
                        <div class="loading-placeholder" style="height:96px" />
                    </Show>
                    {move || {
                        let rows = context
                            .artifacts
                            .data
                            .with(|value| {
                                value
                                    .as_array()
                                    .into_iter()
                                    .flatten()
                                    .filter(|row| {
                                        api::text(row, "project_job_id") == id.get_value()
                                    })
                                    .cloned()
                                    .collect::<Vec<_>>()
                            });
                        if rows.is_empty() && !context.artifacts.loading.get()
                            && context.artifacts.error.get().is_empty()
                        {
                            view! {
                                <Notice
                                    title="No artifacts available"
                                    message=messages::empty_artifacts(&job.get_value())
                                />
                            }
                                .into_any()
                        } else {
                            rows.into_iter()
                                .map(|row| {
                                    let artifact = api::text(&row, "id");
                                    let filename = api::text(&row, "file_name");
                                    let name = api::text(&row, "name");
                                    let title = if !filename.is_empty() {
                                        filename.clone()
                                    } else if !name.is_empty() {
                                        name
                                    } else {
                                        "artifact".into()
                                    };
                                    view! {
                                        <div class="pipeline-artifact pipeline-stack">
                                            <strong>{title.clone()}</strong>
                                            <small>
                                                {model::bytes(
                                                    row["byte_size"].as_u64().unwrap_or_default(),
                                                )}
                                            </small>
                                            <span class="pipeline-download">
                                                <thaw::Icon
                                                    icon=icondata_lu::LuDownload
                                                    width="14px"
                                                    height="14px"
                                                />
                                                <crate::files::Download
                                                    path=format!(
                                                        "{}/jobs/{}/artifacts/{artifact}",
                                                        context.path.get_value(),
                                                        id.get_value(),
                                                    )
                                                    filename=title
                                                />
                                            </span>
                                        </div>
                                    }
                                })
                                .collect_view()
                                .into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
