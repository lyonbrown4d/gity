use super::*;
use serde_json::json;
#[component]
pub(super) fn Locks(query: Query, path: String, can_write: Signal<bool>) -> impl IntoView {
    let lock_path = RwSignal::new(String::new());
    let creation = Mutation::new(query.refresh);
    let unlocking = RwSignal::new(false);
    let delete_path = StoredValue::new(path.clone());
    let submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if !can_write.get_untracked() || creation.busy.get_untracked() {
            return;
        }
        let normalized = lock_path.get_untracked().trim().to_string();
        if normalized.is_empty() {
            creation.error.set(t("LFS lock path is required"));
            return;
        }
        creation.run(
            "POST",
            format!("{path}/lfs/locks"),
            json!({"path":normalized}),
            Some(Callback::new(move |_| lock_path.set(String::new()))),
        );
    };
    view! {
        <section class="lfs-box">
            <h3>
                <thaw::Icon icon=icondata_lu::LuLock width="16px" height="16px" />
                {t("File locks")}
            </h3>
            <div class="lfs-lock-body">
                <form class="lfs-lock-form" on:submit=submit>
                    <label for="lfs-lock-path">
                        {t("Lock path")}
                        <input
                            id="lfs-lock-path"
                            placeholder="assets/design.psd"
                            disabled=move || !can_write.get()
                            prop:value=move || lock_path.get()
                            on:input=move |event| lock_path.set(event_target_value(&event))
                        />
                    </label>
                    <button
                        class="primary"
                        type="submit"
                        disabled=move || !can_write.get() || creation.busy.get()
                    >
                        <thaw::Icon icon=icondata_lu::LuLock width="16px" height="16px" />
                        {move || t(if creation.busy.get() { "Locking..." } else { "Create lock" })}
                    </button>
                </form>
                <Show when=move || !creation.error.get().is_empty()>
                    <p class="error" role="alert">
                        {move || creation.error.get()}
                    </p>
                </Show>
                <Show when=move || query.loading.get()>
                    <p class="muted">{t("Loading LFS locks...")}</p>
                </Show>
                <Show when=move || {
                    !query.loading.get() && query.error.get().is_empty()
                        && rows(query, "locks").is_empty()
                }>
                    <Empty
                        title="No file locks."
                        description=Signal::derive(move || t(
                            if can_write.get() {
                                "Create a lock to reserve an LFS-managed path before editing."
                            } else {
                                "Active file locks will appear here."
                            },
                        ))
                    />
                </Show>
                {move || {
                    rows(query, "locks")
                        .into_iter()
                        .map(|row| {
                            let path = api::text(&row, "path");
                            let owner = api::text(&row["owner"], "name");
                            let date = api::text(&row, "locked_at");
                            view! {
                                <div class="lfs-lock">
                                    <div>
                                        <div>
                                            <strong>{path.clone()}</strong>
                                            <span class="badge secondary">
                                                {if owner.is_empty() { t("Unknown owner") } else { owner }}
                                            </span>
                                        </div>
                                        <small>
                                            {format!(
                                                "#{}{}",
                                                api::text(&row, "id"),
                                                if date.is_empty() {
                                                    String::new()
                                                } else {
                                                    format!(" · {}", relative_time(&date))
                                                },
                                            )}
                                        </small>
                                    </div>
                                    <Action
                                        label=t("Unlock")
                                        path=format!(
                                            "{}/lfs/locks/{}/unlock",
                                            delete_path.get_value(),
                                            api::enc(&api::text(&row, "id")),
                                        )
                                        body=json!({"force":true})
                                        refresh=query.refresh
                                        busy_state=unlocking
                                        confirm=true
                                        disabled=Signal::derive(move || !can_write.get())
                                        confirmation_title=t("Unlock \"{path}\"?")
                                            .replace("{path}", &path)
                                        confirmation_description=t(
                                            "This force-unlocks the file for the project.",
                                        )
                                        confirmation_label=t("Unlock")
                                    >
                                        <thaw::Icon
                                            icon=icondata_lu::LuLockOpen
                                            width="16px"
                                            height="16px"
                                        />
                                        {t("Unlock")}
                                    </Action>
                                </div>
                            }
                        })
                        .collect_view()
                }}
            </div>
        </section>
    }
}
