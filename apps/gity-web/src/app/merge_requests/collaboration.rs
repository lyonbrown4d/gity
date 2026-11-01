use super::{detail::Review, *};
use serde_json::json;

#[component]
pub(super) fn Participants(review: Review) -> impl IntoView {
    view! {
        <div class="mr-box mr-stack">
            <div class="mr-between">
                <div>
                    <div class="mr-tags">
                        <thaw::Icon icon=icondata_lu::LuUserRound width="16px" height="16px" />
                        <strong>{t("Reviewers and assignees")}</strong>
                    </div>
                    <p class="mr-muted mr-small">
                        {t("Assign people who should review or own this merge request.")}
                    </p>
                </div>
                <Reload q=review.participants />
            </div>
            <Status q=review.participants />
            <Status q=review.context.users />
            <div class="mr-participant-roles">
                <Role
                    review=review
                    role="reviewer"
                    label="Reviewers"
                    empty="No reviewers assigned."
                />
                <Role
                    review=review
                    role="assignee"
                    label="Assignees"
                    empty="No assignees assigned."
                />
            </div>
        </div>
    }
}
#[component]
fn Role(
    review: Review,
    role: &'static str,
    label: &'static str,
    empty: &'static str,
) -> impl IntoView {
    let draft = RwSignal::new(String::new());
    let mutation = Mutation::new(review.participants.refresh);
    let save = Callback::new(move |ids: Vec<String>| {
        mutation.run(
            "PATCH",
            format!(
                "{}/{}",
                review.path.get_value(),
                if role == "reviewer" {
                    "reviewers"
                } else {
                    "assignees"
                }
            ),
            json!({"user_ids":model::ids(&ids)}),
            Some(Callback::new(move |_| {
                draft.set(String::new());
                review.checks.refresh.update(|n| *n += 1);
            })),
        );
    });
    view! {
        <div class="mr-box mr-stack mr-role">
            <strong>{t(label)}</strong>
            <div class="mr-tags">
                <Show when=move || review.people(role).is_empty()>
                    <span class="mr-muted mr-small">{t(empty)}</span>
                </Show>
                {move || {
                    review
                        .people(role)
                        .into_iter()
                        .map(|person| {
                            let id = api::text(&person, "user_id");
                            let remove = id.clone();
                            view! {
                                <span class="mr-badge">
                                    {model::user(&review.context.users.data.get(), &id)}
                                    <button
                                        type="button"
                                        class="mr-remove"
                                        aria-label=format!(
                                            "{} {}",
                                            t("Remove"),
                                            model::user(&review.context.users.data.get(), &id),
                                        )
                                        disabled=move || {
                                            mutation.busy.get() || !review.context.can("write", 30)
                                        }
                                        on:click=move |_| {
                                            save.run(
                                                review
                                                    .people(role)
                                                    .iter()
                                                    .map(|p| api::text(p, "user_id"))
                                                    .filter(|id| *id != remove)
                                                    .collect(),
                                            );
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
                    aria-label=t(label)
                    disabled=move || !review.context.can("write", 30) || mutation.busy.get()
                    prop:value=move || draft.get()
                    on:change=move |e| draft.set(event_target_value(&e))
                >
                    <option value="">{t("Select user")}</option>
                    {move || {
                        api::list(&review.context.users.data.get())
                            .into_iter()
                            .filter(|user| {
                                !review
                                    .people(role)
                                    .iter()
                                    .any(|p| api::text(p, "user_id") == api::text(user, "id"))
                            })
                            .map(|user| {
                                let id = api::text(&user, "id");
                                view! {
                                    <option value=id
                                        .clone()>
                                        {model::user(&review.context.users.data.get(), &id)}
                                    </option>
                                }
                            })
                            .collect_view()
                    }}
                </select>
                <button
                    type="button"
                    disabled=move || {
                        mutation.busy.get() || draft.get().is_empty()
                            || !review.context.can("write", 30)
                    }
                    on:click=move |_| {
                        let mut ids = review
                            .people(role)
                            .iter()
                            .map(|p| api::text(p, "user_id"))
                            .collect::<Vec<_>>();
                        ids.push(draft.get_untracked());
                        ids.sort();
                        ids.dedup();
                        save.run(ids);
                    }
                >
                    {move || t(if mutation.busy.get() { "Saving..." } else { "Add" })}
                </button>
            </div>
            <MutationError mutation=mutation />
        </div>
    }
}

#[component]
pub(super) fn Approvals(review: Review) -> impl IntoView {
    let mutation = Mutation::new(review.approvals.refresh);
    view! {
        <div class="mr-box mr-stack">
            <div class="mr-between">
                <div>
                    <div class="mr-tags">
                        <thaw::Icon icon=icondata_lu::LuThumbsUp width="16px" height="16px" />
                        <strong>{t("Approvals")}</strong>
                    </div>
                    <p class="mr-muted mr-small">
                        {t("Track reviewers who approved this merge request.")}
                    </p>
                </div>
                <div class="mr-tags">
                    <Reload q=review.approvals />
                    <button
                        type="button"
                        class:primary=move || !review.approved()
                        disabled=move || {
                            !review.context.can("write", 30)
                                || api::text(&review.mr.get_value(), "state") != "opened"
                                || mutation.busy.get() || review.approvals.loading.get()
                        }
                        on:click=move |_| {
                            mutation
                                .run(
                                    "POST",
                                    format!(
                                        "{}/{}",
                                        review.path.get_value(),
                                        if review.approved() { "unapprove" } else { "approve" },
                                    ),
                                    json!({}),
                                    Some(
                                        Callback::new(move |_| {
                                            review.checks.refresh.update(|n| *n += 1)
                                        }),
                                    ),
                                );
                        }
                    >
                        {move || t(
                            if mutation.busy.get() {
                                "Saving..."
                            } else if review.approved() {
                                "Unapprove"
                            } else {
                                "Approve"
                            },
                        )}
                    </button>
                </div>
            </div>
            <Status q=review.approvals />
            <MutationError mutation=mutation />
            <div class="mr-tags">
                <Show when=move || {
                    model::list(&review.approvals.data.get(), "approvals").is_empty()
                }>
                    <span class="mr-small mr-muted">{t("No approvals yet.")}</span>
                </Show>
                {move || {
                    model::list(&review.approvals.data.get(), "approvals")
                        .into_iter()
                        .map(|approval| {
                            view! {
                                <span class="mr-badge">
                                    <thaw::Icon
                                        icon=icondata_lu::LuCircleCheck
                                        width="12px"
                                        height="12px"
                                    />
                                    {model::user(
                                        &review.context.users.data.get(),
                                        &api::text(&approval, "user_id"),
                                    )}
                                </span>
                            }
                        })
                        .collect_view()
                }}
            </div>
        </div>
    }
}
