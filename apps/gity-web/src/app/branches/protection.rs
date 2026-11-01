use super::BranchMutation;
use crate::{api, i18n::t};
use leptos::prelude::*;
use serde_json::{Value, json};

#[component]
pub(super) fn Protection(
    value: Value,
    project_path: String,
    branch: String,
    can_admin: Signal<bool>,
    mutation: BranchMutation,
) -> impl IntoView {
    let path = format!(
        "{project_path}/repository/branch-protections/{}",
        api::enc(&branch)
    );
    let current = StoredValue::new(value.clone());
    let update = Callback::new(move |(key, next): (&'static str, Value)| {
        if !can_admin.get_untracked() {
            return;
        }
        let mut body = current.get_value();
        body[key] = next;
        mutation.run("PATCH", path.clone(), body, None);
    });
    view! {
        <p>
            {t("Push")}": "{t(&api::text(&value, "push_access_level"))}" / "{t("Merge")}": "
            {t(&api::text(&value, "merge_access_level"))} " / "
            {t(if value["allow_delete"] == true { "Deletion allowed" } else { "Deletion blocked" })}
        </p>
        <div class="branch-protection">
            {[("push_access_level", "Push access"), ("merge_access_level", "Merge access")]
                .into_iter()
                .map(|(key, label)| {
                    let selected = api::text(&value, key);
                    view! {
                        <label>
                            {t(label)}
                            <select
                                aria-label=t(label)
                                disabled=move || !can_admin.get() || mutation.busy.get()
                                prop:value=selected
                                on:change=move |event| {
                                    update.run((key, json!(event_target_value(&event))))
                                }
                            >
                                {[
                                    ("no_one", "No one"),
                                    ("developer", "Developer"),
                                    ("maintainer", "Maintainer"),
                                    ("owner", "Owner"),
                                ]
                                    .into_iter()
                                    .map(|(key, label)| {
                                        view! { <option value=key>{t(label)}</option> }
                                    })
                                    .collect_view()}
                            </select>
                        </label>
                    }
                })
                .collect_view()}
            <div>
                {[
                    ("require_merge_request", "Require merge request"),
                    ("require_pipeline_success", "Require successful pipeline"),
                    ("allow_force_push", "Allow force push"),
                    ("allow_delete", "Allow delete"),
                ]
                    .into_iter()
                    .map(|(key, label)| {
                        let enabled = value[key].as_bool().unwrap_or(false);
                        view! {
                            <button
                                class:primary=enabled
                                disabled=move || !can_admin.get() || mutation.busy.get()
                                aria-pressed=enabled.to_string()
                                on:click=move |_| update.run((key, json!(!enabled)))
                            >
                                {t(label)}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>
        </div>
    }
}
