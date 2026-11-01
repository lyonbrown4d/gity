use super::*;

#[component]
pub(super) fn Sidebar(context: Context) -> impl IntoView {
    view! {
        <aside class="releases-sidebar">
            <div class="releases-box releases-stack">
                <strong class="release-section-title">{t("Tags")}</strong>
                <Status q=context.tags />
                <div class="release-tag-list">
                    <Show when=move || {
                        !context.tags.loading.get() && context.tags.error.get().is_empty()
                            && api::list(&context.tags.data.get()).is_empty()
                    }>
                        <Empty
                            title=t("No tags yet.")
                            description=t("Tags created for releases will appear here.")
                        />
                    </Show>
                    {move || {
                        api::list(&context.tags.data.get())
                            .into_iter()
                            .map(|tag| view! { <Tag context=context tag=tag /> })
                            .collect_view()
                    }}
                </div>
            </div>
            <div class="releases-box releases-stack">
                <strong class="release-section-title">{t("Selected release assets")}</strong>
                <Show when=move || context.active().is_null()>
                    <Empty
                        title=t("Select a release.")
                        description=t("Release asset links are shown for the selected release.")
                    />
                </Show>
                <Show when=move || {
                    !context.active().is_null() && model::links(&context.active()).is_empty()
                }>
                    {move || {
                        view! {
                            <Empty
                                title=t("No asset links.")
                                description=t(
                                    if context.can_admin() {
                                        "Add an asset link for downloadable artifacts or runbooks."
                                    } else {
                                        "Asset links will appear here after they are added."
                                    },
                                )
                            />
                        }
                    }}
                </Show>
                <div class="release-tag-list">
                    {move || {
                        let selected = context.active();
                        let release_id = api::text(&selected["release"], "id");
                        model::links(&selected)
                            .into_iter()
                            .map(|link| {
                                view! {
                                    <Asset
                                        context=context
                                        release_id=release_id.clone()
                                        link=link
                                    />
                                }
                            })
                            .collect_view()
                    }}
                </div>
            </div>
        </aside>
    }
}
#[component]
fn Tag(context: Context, tag: Value) -> impl IntoView {
    let name = api::text(&tag, "name");
    view! {
        <div class="release-tag releases-between">
            <div>
                <strong class="releases-actions">
                    <thaw::Icon icon=icondata_lu::LuTag width="14px" height="14px" />
                    {name.clone()}
                </strong>
                <code>{model::short_sha(&api::text(&tag, "target_sha"))}</code>
            </div>
            <Show when=move || context.can_admin()>
                <Remove
                    label=t("Delete tag")
                    title=t("Delete tag \"{name}\"?").replace("{name}", &name)
                    description=t("Deleting a Git tag can affect release consumers.")
                    path=format!(
                        "{}/repository/tags?name={}",
                        context.path.get_value(),
                        api::enc(&name),
                    )
                    refresh=context.tags.refresh
                    can_admin=Signal::derive(move || context.can_admin())
                />
            </Show>
        </div>
    }
}
#[component]
fn Asset(context: Context, release_id: String, link: Value) -> impl IntoView {
    let name = api::text(&link, "name");
    let url = model::asset_url(&api::text(&link, "url"));
    view! {
        <div class="release-asset">
            {url
                .map(|url| {
                    view! {
                        <a href=url target="_blank" rel="noopener noreferrer">
                            <thaw::Icon
                                icon=icondata_lu::LuExternalLink
                                width="14px"
                                height="14px"
                            />
                            {name.clone()}
                        </a>
                    }
                        .into_any()
                })
                .unwrap_or_else(|| view! { <span>{name.clone()}</span> }.into_any())}
            <span class="releases-badge">{api::text(&link, "link_type")}</span>
            <Show when=move || context.can_admin()>
                <Remove
                    label=t("Delete asset link")
                    title=t("Delete asset link")
                    description=String::new()
                    path=format!(
                        "{}/releases/{release_id}/links/{}",
                        context.path.get_value(),
                        api::text(&link, "id"),
                    )
                    refresh=context.tags.refresh
                    can_admin=Signal::derive(move || context.can_admin())
                    confirm=false
                />
            </Show>
        </div>
    }
}
