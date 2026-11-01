use super::*;
use serde_json::json;

pub(super) const DEFAULT_CONFIG: &str = "stages:\n  - test\n\njobs:\n  test:\n    stage: test\n    image: rust:1\n    script:\n      - cargo test\n";

#[component]
pub(super) fn Composer(context: Context) -> impl IntoView {
    let create = Mutation::new(context.updates);
    let lint = Mutation::new(RwSignal::new(0));
    let lint_config = move |_| {
        if !context.can_write() {
            return;
        }
        context.lint_result.set(String::new());
        lint.run_with_result(
            "POST",
            format!("{}/ci/lint", context.path.get_value()),
            json!({"config_content":context.content.get_untracked()}),
            Some(Callback::new(move |value: Value| {
                context
                    .lint_result
                    .set(serde_json::to_string_pretty(&value).unwrap_or_default())
            })),
        );
    };
    view! {
        <form
            class="pipeline-box pipeline-stack pipeline-composer"
            on:submit=move |event| {
                event.prevent_default();
                if !context.can_write() || lint.busy.get_untracked() {
                    return;
                }
                let body = match model::create(
                    &context.source.get_untracked(),
                    &context.reference.get_untracked(),
                    &context.commit.get_untracked(),
                    &context.config.get_untracked(),
                    &context.content.get_untracked(),
                ) {
                    Ok(body) => body,
                    Err(error) => {
                        create.error.set(t(error));
                        return;
                    }
                };
                create
                    .run_with_result(
                        "POST",
                        format!("{}/pipelines", context.path.get_value()),
                        body,
                        Some(
                            Callback::new(move |value: Value| {
                                context.select(api::text(&value["pipeline"], "id"));
                                context.lint_result.set(String::new());
                                context.composer.set(false);
                            }),
                        ),
                    );
            }
        >
            <div class="pipeline-form-fields">
                <label>
                    {t("Source")}
                    <select
                        aria-label=t("Source")
                        prop:value=move || context.source.get()
                        on:change=move |event| context.source.set(event_target_value(&event))
                    >
                        <option value="web">"web"</option>
                        <option value="manual">"manual"</option>
                        <option value="push" disabled>
                            "push"
                        </option>
                        <option value="merge_request">"merge_request"</option>
                    </select>
                </label>
                <label>
                    {t("Ref")}
                    <input
                        aria-label=t("Ref")
                        required
                        prop:value=move || context.reference.get()
                        on:input=move |event| context.reference.set(event_target_value(&event))
                    />
                </label>
                <label>
                    {t("Commit SHA")}
                    <input
                        aria-label=t("Commit SHA")
                        placeholder=t("Optional")
                        prop:value=move || context.commit.get()
                        on:input=move |event| context.commit.set(event_target_value(&event))
                    />
                </label>
            </div>
            <label>
                {t("Config source")}
                <input
                    aria-label=t("Config source")
                    prop:value=move || context.config.get()
                    on:input=move |event| context.config.set(event_target_value(&event))
                />
            </label>
            <label>{t("YAML CI config")}</label>
            <crate::editors::CodeEditor
                id="pipeline-config".into()
                name="config_content".into()
                content=context.content.get_untracked()
                language="yaml".into()
                on_change=Callback::new(move |value: String| context.content.set(value))
            />
            <Show when=move || !context.lint_result.get().is_empty()>
                <pre class="pipeline-lint">{move || context.lint_result.get()}</pre>
            </Show>
            <MutationError mutation=lint />
            <MutationError mutation=create />
            <div class="pipeline-end">
                <button
                    type="button"
                    disabled=move || !context.can_write() || create.busy.get() || lint.busy.get()
                    on:click=lint_config
                >
                    {move || t(if lint.busy.get() { "Linting..." } else { "Lint config" })}
                </button>
                <button
                    type="submit"
                    class="primary"
                    disabled=move || !context.can_write() || create.busy.get() || lint.busy.get()
                >
                    <thaw::Icon icon=icondata_lu::LuPlay width="14px" height="14px" />
                    {move || t(
                        if create.busy.get() { "Creating pipeline..." } else { "Create pipeline" },
                    )}
                </button>
            </div>
        </form>
    }
}
