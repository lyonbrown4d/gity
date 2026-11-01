use super::{rules::RulesState, *};
use serde_json::json;

#[component]
pub(super) fn RuleForm(context: Context, state: RulesState) -> impl IntoView {
    let editing = state.editing.get_untracked();
    let id = api::text(&editing, "id");
    let is_editing = !id.is_empty();
    let name = RwSignal::new(api::text(&editing, "name"));
    let branch = api::text(&editing, "target_branch");
    let default_branch = api::text(&context.project.data.get_untracked(), "default_branch");
    let initialized = RwSignal::new(!branch.is_empty() || !default_branch.is_empty());
    let target = RwSignal::new(if branch.is_empty() {
        if default_branch.is_empty() {
            "*".into()
        } else {
            default_branch
        }
    } else {
        branch
    });
    Effect::new(move |_| {
        let branch = api::text(&context.project.data.get(), "default_branch");
        if !initialized.get_untracked() && !branch.is_empty() {
            target.set(branch);
            initialized.set(true);
        }
    });
    let required = RwSignal::new(editing["approvals_required"].as_i64().unwrap_or(1));
    let eligible = RwSignal::new(model::strings(&editing["eligible_user_ids"]));
    let draft = RwSignal::new(String::new());
    let owners = RwSignal::new(editing["code_owner"] == true);
    let mutation = Mutation::new(state.query.refresh);
    let disabled = Signal::derive(move || !context.can("merge", 40) || mutation.busy.get());
    view! {
        <form
            class="mr-box mr-stack mr-rule-form"
            on:submit=move |e| {
                e.prevent_default();
                if disabled.get_untracked() || name.get_untracked().trim().is_empty() {
                    return;
                }
                let path = format!(
                    "{}/merge-request-approval-rules{}",
                    context.path.get_value(),
                    if is_editing { format!("/{id}") } else { String::new() },
                );
                mutation
                    .run(
                        if is_editing { "PATCH" } else { "POST" },
                        path,
                        json!(
                            {
                "name":name.get_untracked().trim(), "target_branch":target.get_untracked(),
                "approvals_required":required.get_untracked().max(1), "eligible_user_ids":model::ids(&eligible.get_untracked()),
                "code_owner":owners.get_untracked()
            }
                        ),
                        Some(Callback::new(move |_| state.reset())),
                    );
            }
        >
            <div class="mr-rule-fields">
                <label>
                    <span>{t("Rule name")}</span>
                    <input
                        aria-label=t("Rule name")
                        placeholder=t("Maintainer approval")
                        disabled=move || disabled.get()
                        prop:value=move || name.get()
                        on:input=move |e| name.set(event_target_value(&e))
                        required
                    />
                </label>
                <composer::BranchSelect
                    context=context
                    label="Target branch"
                    selected=target
                    all=true
                    disabled=disabled
                />
                <label>
                    <span>{t("Required")}</span>
                    <input
                        aria-label=t("Required")
                        type="number"
                        min="1"
                        disabled=move || disabled.get()
                        prop:value=move || required.get()
                        on:input=move |e| required.set(event_target_value(&e).parse().unwrap_or(1))
                    />
                </label>
            </div>
            <div class="mr-rule-people">
                <div class="mr-stack">
                    <label>{t("Eligible users")}</label>
                    <div class="mr-tags">
                        <Show when=move || eligible.get().is_empty()>
                            <small class="mr-muted">
                                {t("Any approver can satisfy this rule.")}
                            </small>
                        </Show>
                        {move || {
                            eligible
                                .get()
                                .into_iter()
                                .map(|id| {
                                    let remove = id.clone();
                                    view! {
                                        <span class="mr-badge">
                                            {model::user(&context.users.data.get(), &id)}
                                            <button
                                                type="button"
                                                class="mr-remove"
                                                aria-label=format!(
                                                    "{} {}",
                                                    t("Remove"),
                                                    model::user(&context.users.data.get(), &id),
                                                )
                                                disabled=move || disabled.get()
                                                on:click=move |_| {
                                                    eligible.update(|ids| ids.retain(|id| *id != remove))
                                                }
                                            >
                                                "×"
                                            </button>
                                        </span>
                                    }
                                })
                                .collect_view()
                        }}
                    </div>
                    <div class="mr-add-person">
                        <select
                            aria-label=t("Select eligible user")
                            disabled=move || disabled.get()
                            prop:value=move || draft.get()
                            on:change=move |e| draft.set(event_target_value(&e))
                        >
                            <option value="">{t("Select eligible user")}</option>
                            {move || {
                                api::list(&context.users.data.get())
                                    .into_iter()
                                    .filter(|user| !eligible.get().contains(&api::text(user, "id")))
                                    .map(|user| {
                                        let id = api::text(&user, "id");
                                        view! {
                                            <option value=id
                                                .clone()>
                                                {model::user(&context.users.data.get(), &id)}
                                            </option>
                                        }
                                    })
                                    .collect_view()
                            }}
                        </select>
                        <button
                            type="button"
                            disabled=move || disabled.get() || draft.get().is_empty()
                            on:click=move |_| {
                                eligible
                                    .update(|ids| {
                                        let id = draft.get_untracked();
                                        if !ids.contains(&id) {
                                            ids.push(id);
                                        }
                                    });
                                draft.set(String::new());
                            }
                        >
                            {t("Add user")}
                        </button>
                    </div>
                </div>
                <div class="mr-stack mr-rule-buttons">
                    <button
                        type="button"
                        class:mr-selected=move || owners.get()
                        disabled=move || disabled.get()
                        on:click=move |_| owners.update(|v| *v = !*v)
                    >
                        {move || t(
                            if owners.get() { "CODEOWNERS required" } else { "CODEOWNERS optional" },
                        )}
                    </button>
                    <button
                        type="submit"
                        class="primary"
                        disabled=move || disabled.get() || name.get().trim().is_empty()
                    >
                        {move || t(
                            if mutation.busy.get() {
                                "Saving..."
                            } else if is_editing {
                                "Update rule"
                            } else {
                                "Create rule"
                            },
                        )}
                    </button>
                    <Show when=move || is_editing>
                        <button type="button" class="mr-ghost" on:click=move |_| state.reset()>
                            {t("Cancel edit")}
                        </button>
                    </Show>
                </div>
            </div>
            <MutationError mutation=mutation />
        </form>
    }
}
