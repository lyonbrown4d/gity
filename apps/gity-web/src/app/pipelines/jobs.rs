use super::*;

#[component]
pub(super) fn Jobs(context: Context) -> impl IntoView {
    view! {
        <div class="pipeline-stack pipeline-jobs">
            <h3>{t("Pipeline jobs")}</h3>
            <Show when=move || context.detail.loading.get()>
                <div class="loading-placeholder" style="height:96px" />
                <div class="loading-placeholder" style="height:96px" />
            </Show>
            <Show when=move || {
                !context.detail.loading.get() && context.detail.error.get().is_empty()
                    && context.jobs.get().is_empty()
            }>
                <Notice
                    title="No jobs in this pipeline"
                    message="This pipeline has no materialized jobs yet. Refresh the pipeline state after workers process the config."
                />
            </Show>
            <For
                each=move || context.jobs.get()
                key=|job| (
                    job.id.clone(),
                    job.state,
                    job.updated.clone(),
                    job.attempts.clone(),
                    job.error.clone(),
                )
                children=move |job| view! { <JobCard context=context job=job /> }
            />
        </div>
    }
}
#[component]
fn JobCard(context: Context, job: Job) -> impl IntoView {
    let id = StoredValue::new(job.id.clone());
    let state = job.state;
    let underlying = job.underlying;
    let path = format!("{}/jobs/{}", context.path.get_value(), job.id);
    let worker = if job.worker.is_empty() {
        String::new()
    } else {
        format!(" · {}: {}", t("Worker"), job.worker)
    };
    view! {
        <div
            class="pipeline-job pipeline-stack"
            class:selected=move || context.selected_job.get() == id.get_value()
            class:failed=state == State::Failed
        >
            <div class="pipeline-between">
                <div class="pipeline-job-heading">
                    <div class="pipeline-actions">
                        <strong>{job.title()}</strong>
                        <Badge state=state />
                    </div>
                    <p>{t(messages::description(state))}</p>
                    <p>
                        {format!(
                            "{}: {}/{}{}",
                            t("Attempts"),
                            job.attempts,
                            job.max_attempts,
                            worker,
                        )}
                        {if job.updated.is_empty() {
                            String::new()
                        } else {
                            format!(" · {}", relative_time(&job.updated))
                        }}
                    </p>
                </div>
                <div class="pipeline-actions">
                    <span class="pipeline-badge">
                        {if job.stage.is_empty() { t("N/A") } else { job.stage.clone() }}
                    </span>
                    <button
                        type="button"
                        class="pipeline-ghost"
                        on:click=move |_| context.selected_job.set(id.get_value())
                    >
                        <thaw::Icon icon=icondata_lu::LuScrollText width="14px" height="14px" />
                        {t("Logs")}
                    </button>
                    <Show when=move || !underlying.terminal()>
                        <Action
                            label=t("Cancel")
                            path=format!(
                                "{}/jobs/{}/cancel",
                                context.path.get_value(),
                                id.get_value(),
                            )
                            refresh=context.updates
                            busy_state=context.cancelling_job
                            disabled=Signal::derive(move || !context.can_write())
                        >
                            <thaw::Icon icon=icondata_lu::LuBan width="14px" height="14px" />
                            {move || t(
                                if context.cancelling_job.get() {
                                    "Cancelling..."
                                } else {
                                    "Cancel"
                                },
                            )}
                        </Action>
                    </Show>
                    <Show when=move || underlying != State::Running>
                        <Action
                            label=t("Retry")
                            path=format!("{path}/retry")
                            refresh=context.updates
                            busy_state=context.retrying_job
                            disabled=Signal::derive(move || {
                                !context.can_write() || !underlying.retry_job()
                            })
                        >
                            <thaw::Icon icon=icondata_lu::LuRepeat2 width="14px" height="14px" />
                            {move || t(
                                if context.retrying_job.get() { "Retrying..." } else { "Retry" },
                            )}
                        </Action>
                    </Show>
                </div>
            </div>
            {(!job.needs.is_empty())
                .then(|| {
                    view! {
                        <p class="pipeline-muted">
                            {format!("{}: {}", t("Needs"), job.needs.join(", "))}
                        </p>
                    }
                })}
            {(!job.tags.is_empty())
                .then(|| {
                    view! {
                        <div class="pipeline-actions">
                            {job
                                .tags
                                .iter()
                                .map(|tag| {
                                    view! { <span class="pipeline-badge">{tag.clone()}</span> }
                                })
                                .collect_view()}
                        </div>
                    }
                })}
            {(!job.script.is_empty())
                .then(|| view! { <pre class="pipeline-script">{job.script.join("\n")}</pre> })}
            {(!job.artifacts.is_empty())
                .then(|| {
                    view! {
                        <p class="pipeline-muted">
                            {format!("{}: {}", t("Artifacts"), job.artifacts.join(", "))}
                        </p>
                    }
                })}
            {matches!(state, State::Failed | State::Blocked)
                .then(|| {
                    view! {
                        <aside
                            class="pipeline-notice pipeline-job-notice"
                            class:destructive=state == State::Failed
                        >
                            <p>{t(messages::next_job(&job))}</p>
                        </aside>
                    }
                })}
            {(!job.error.is_empty())
                .then(|| {
                    view! {
                        <aside class="pipeline-notice destructive pipeline-job-notice">
                            <p>{job.error.clone()}</p>
                        </aside>
                    }
                })}
        </div>
    }
}
