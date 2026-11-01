use super::NavigationIcon;
use leptos::prelude::*;
#[component]
pub(super) fn ContextGroup(
    label: String,
    links: Vec<(String, String)>,
    active: String,
    #[prop(default = false)] collapsible: bool,
) -> impl IntoView {
    view! {
        <details class="context-group" class:group-collapsible=collapsible open>
            <summary on:click=move |e| {
                if !collapsible {
                    e.prevent_default();
                }
            }>{crate::i18n::t(&label)}</summary>
            <nav>
                {links
                    .into_iter()
                    .map(|(label, url)| {
                        view! {
                            <a class:active=url == active href=url>
                                <NavigationIcon title=label.clone() />
                                {crate::i18n::t(&label)}
                            </a>
                        }
                    })
                    .collect_view()}
            </nav>
        </details>
    }
}
