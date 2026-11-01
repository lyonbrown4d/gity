use super::*;

#[component]
pub(super) fn Register(context: Context) -> impl IntoView {
    let name = context.draft_name;
    let description = context.draft_description;
    let tags = context.draft_tags;
    let mutation = Mutation::new(context.rows.refresh);
    view! {
        <form
            class="runner-box runner-stack runner-register"
            on:submit=move |event| {
                event.prevent_default();
                if !context.can_admin() {
                    return;
                }
                let body = match model::registration(
                    &name.get_untracked(),
                    &description.get_untracked(),
                    &tags.get_untracked(),
                ) {
                    Ok(body) => body,
                    Err(error) => {
                        mutation.error.set(t(error));
                        return;
                    }
                };
                mutation
                    .run_with_result(
                        "POST",
                        format!("{}/runners", context.path.get_value()),
                        body,
                        Some(
                            Callback::new(move |response: Value| {
                                context.token.set(api::text(&response, "token"));
                                name.set(String::new());
                                description.set(String::new());
                                tags.set("linux,go".into());
                                context.composer.set(false);
                            }),
                        ),
                    );
            }
        >
            <div class="runner-register-fields">
                <label>
                    {t("Runner name")}
                    <input
                        aria-label=t("Runner name")
                        placeholder="linux-amd64"
                        prop:value=move || name.get()
                        on:input=move |e| name.set(event_target_value(&e))
                    />
                </label>
                <label>
                    {t("Runner tags")}
                    <input
                        aria-label=t("Runner tags")
                        placeholder="linux,go"
                        prop:value=move || tags.get()
                        on:input=move |e| tags.set(event_target_value(&e))
                    />
                </label>
            </div>
            <label>
                {t("Description")}
                <input
                    aria-label=t("Description")
                    placeholder=t("Runner description optional")
                    prop:value=move || description.get()
                    on:input=move |e| description.set(event_target_value(&e))
                />
            </label>
            <MutationError mutation=mutation />
            <div class="runner-end">
                <button
                    type="submit"
                    class="primary"
                    disabled=move || !context.can_admin() || mutation.busy.get()
                >
                    {move || t(
                        if mutation.busy.get() {
                            "Registering runner..."
                        } else {
                            "Register runner"
                        },
                    )}
                </button>
            </div>
        </form>
    }
}
