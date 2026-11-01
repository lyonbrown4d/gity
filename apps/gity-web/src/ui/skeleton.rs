use leptos::prelude::*;

/// Shimmering placeholder block reused by pages while their query is pending.
#[component]
pub fn SkeletonLines(
  #[prop(default = 3)] lines: usize,
  #[prop(optional)] class: String,
) -> impl IntoView {
  let blocks = (0..lines)
    .map(|index| {
      let class = if index + 1 == lines {
        "gity-skeleton-line short"
      } else {
        "gity-skeleton-line"
      };
      view! { <thaw::SkeletonItem class=class.to_string() /> }
    })
    .collect_view();
  view! { <thaw::Skeleton class=format!("gity-skeleton {class}")>{blocks}</thaw::Skeleton> }
}

/// Placeholder grid for card lists such as projects or releases.
#[component]
pub fn SkeletonCards(#[prop(default = 4)] cards: usize) -> impl IntoView {
  let blocks = (0..cards)
    .map(|_| view! { <thaw::SkeletonItem class="gity-skeleton-card".to_string() /> })
    .collect_view();
  view! { <thaw::Skeleton class="gity-skeleton grid">{blocks}</thaw::Skeleton> }
}

/// Placeholder rows for tables and dense lists.
#[component]
pub fn SkeletonRows(#[prop(default = 5)] rows: usize) -> impl IntoView {
  let blocks = (0..rows)
    .map(|_| view! { <thaw::SkeletonItem class="gity-skeleton-row".to_string() /> })
    .collect_view();
  view! { <thaw::Skeleton class="gity-skeleton">{blocks}</thaw::Skeleton> }
}
