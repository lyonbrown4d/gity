use super::*;

#[component]
pub(super) fn Listing(context: Context) -> impl IntoView {
    view! {
        <div class="releases-stack">
            <Status q=context.rows />
            <Show when=move || {
                !context.rows.loading.get() && context.rows.error.get().is_empty()
                    && api::list(&context.rows.data.get()).is_empty()
            }>
                {move || {
                    view! {
                        <Empty
                            title=t("No releases yet.")
                            description=t(
                                if context.can_admin() {
                                    "Create a release from an existing tag or source ref."
                                } else {
                                    "Releases will appear here after maintainers publish them."
                                },
                            )
                        />
                    }
                }}
            </Show>
            {move || {
                api::list(&context.rows.data.get())
                    .into_iter()
                    .map(|item| view! { <ReleaseCard context=context item=item /> })
                    .collect_view()
            }}
        </div>
    }
}
#[component]
fn ReleaseCard(context: Context, item: Value) -> impl IntoView {
    let release = &item["release"];
    let id = api::text(release, "id");
    let active_id = id.clone();
    let select_id = id.clone();
    let heading_id = id.clone();
    let edit = StoredValue::new(release.clone());
    let name = api::text(release, "name");
    let confirm_name = name.clone();
    let links = model::links(&item);
    let has_links = !links.is_empty();
    let description = api::text(release, "description");
    let date = api::text(release, "released_at");
    view! {
        <article
            class="release-card"
            class:selected=move || context.selected.get() == active_id
            on:click=move |_| context.selected.set(select_id.clone())
        >
            <div class="releases-between">
                <div class="release-heading">
                    <div class="releases-actions">
                        <h3>
                            <button
                                type="button"
                                class="release-select"
                                on:click=move |_| context.selected.set(heading_id.clone())
                            >
                                {name.clone()}
                            </button>
                        </h3>
                        <span class="releases-badge releases-secondary">
                            {api::text(release, "tag_name")}
                        </span>
                        {if item["tag"].is_object() {
                            Some(
                                view! {
                                    <span class="releases-badge">
                                        {model::short_sha(&api::text(&item["tag"], "target_sha"))}
                                    </span>
                                }
                                    .into_any(),
                            )
                        } else {
                            Some(
                                view! {
                                    <span class="releases-badge releases-danger">
                                        {t("Missing tag")}
                                    </span>
                                }
                                    .into_any(),
                            )
                        }}
                    </div>
                    <div class="release-notes">
                        {if description.is_empty() {
                            view! { <p>{t("No release notes.")}</p> }.into_any()
                        } else {
                            view! { <crate::markdown::Markdown content=description /> }.into_any()
                        }}
                    </div>
                    <small>
                        {if date.is_empty() { t("No release time") } else { relative_time(&date) }}
                    </small>
                </div>
                <Show when=move || context.can_admin()>
                    <div class="releases-actions">
                        <button
                            type="button"
                            class="releases-ghost release-edit"
                            aria-label=t("Edit release")
                            title=t("Edit release")
                            on:click=move |e| {
                                e.stop_propagation();
                                context.selected.set(api::text(&edit.get_value(), "id"));
                                context.edit(edit.get_value());
                            }
                        >
                            <thaw::Icon icon=icondata_lu::LuPencil width="16px" height="16px" />
                        </button>
                        <Remove
                            label=t("Delete release")
                            title=t("Delete release \"{name}\"?").replace("{name}", &confirm_name)
                            description=t("This removes release metadata but keeps the Git tag.")
                            path=format!("{}/releases/{id}", context.path.get_value())
                            refresh=context.tags.refresh
                            can_admin=Signal::derive(move || context.can_admin())
                        />
                    </div>
                </Show>
            </div>
            <Show when=move || has_links>
                <div class="release-link-badges">
                    {links
                        .iter()
                        .map(|link| {
                            view! {
                                <span class="releases-badge">
                                    <thaw::Icon
                                        icon=icondata_lu::LuLink2
                                        width="12px"
                                        height="12px"
                                    />
                                    {api::text(link, "name")}
                                </span>
                            }
                        })
                        .collect_view()}
                </div>
            </Show>
        </article>
    }
}
