use super::*;
use serde_json::json;

#[component]
pub(super) fn Composer(context: Context) -> impl IntoView {
    let title = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let source = RwSignal::new(String::new());
    let target = RwSignal::new(String::new());
    let mutation = Mutation::new(context.rows.refresh);
    Effect::new(move |_| {
        let branch = api::text(&context.project.data.get(), "default_branch");
        if target.get_untracked().is_empty() {
            target.set(if branch.is_empty() {
                "main".into()
            } else {
                branch
            });
        }
    });
    view! {
        <form
            class="mr-box mr-stack"
            on:submit=move |e| {
                e.prevent_default();
                if !context.can("create", 30) {
                    return;
                }
                if source.get_untracked() == target.get_untracked() {
                    mutation.error.set(t("Source and target branches must differ"));
                    return;
                }
                mutation
                    .run_with_result(
                        "POST",
                        format!("{}/merge-requests", context.path.get_value()),
                        json!(
                            {
                "title":title.get_untracked().trim(), "description":description.get_untracked().trim(),
                "source_branch":source.get_untracked(), "target_branch":target.get_untracked()
            }
                        ),
                        Some(
                            Callback::new(move |value: Value| {
                                context.pending.set(api::text(&value, "iid"));
                                context.composer.set(false);
                                context.reset_filters();
                            }),
                        ),
                    );
            }
        >
            <input
                aria-label=t("Merge request title")
                placeholder=t("Merge request title")
                required
                prop:value=move || title.get()
                on:input=move |e| title.set(event_target_value(&e))
            />
            <textarea
                aria-label=t("Describe the merge request (optional)")
                placeholder=t("Describe the merge request (optional)")
                rows="4"
                prop:value=move || description.get()
                on:input=move |e| description.set(event_target_value(&e))
            ></textarea>
            <div class="mr-branches">
                <BranchSelect context=context label="Source branch" selected=source />
                <BranchSelect context=context label="Target branch" selected=target />
            </div>
            <MutationError mutation=mutation />
            <div class="mr-end">
                <button
                    type="submit"
                    class="primary"
                    disabled=move || {
                        mutation.busy.get() || !context.can("create", 30)
                            || title.get().trim().is_empty() || source.get().is_empty()
                            || target.get().is_empty()
                    }
                >
                    {move || t(
                        if mutation.busy.get() {
                            "Creating merge request..."
                        } else {
                            "Create merge request"
                        },
                    )}
                </button>
            </div>
        </form>
    }
}

#[component]
pub(super) fn BranchSelect(
    context: Context,
    label: &'static str,
    selected: RwSignal<String>,
    #[prop(default = false)] all: bool,
    #[prop(default=Signal::stored(false))] disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <label>
            <span>{t(label)}</span>
            <select
                aria-label=t(label)
                required
                disabled=move || disabled.get()
                prop:value=move || selected.get()
                on:change=move |e| selected.set(event_target_value(&e))
            >
                <option value=if all {
                    "*"
                } else {
                    ""
                }>{t(if all { "All branches" } else { "Select branch" })}</option>
                {move || {
                    api::list(&context.branches.data.get())
                        .into_iter()
                        .map(|row| {
                            let branch = api::text(&row, "name");
                            let value = branch.clone();
                            view! {
                                <option
                                    value=branch.clone()
                                    prop:selected=move || selected.get() == value
                                >
                                    {branch.clone()}
                                </option>
                            }
                        })
                        .collect_view()
                }}
            </select>
        </label>
    }
}
