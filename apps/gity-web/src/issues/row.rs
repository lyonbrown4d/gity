use super::{assignees, labels};
use crate::{api, ui::Query};
use leptos::prelude::*;
use leptos_router::components::A;
use serde_json::Value;
fn user_name(users: Query, id: &str) -> String {
  api::list(&users.data.get())
    .into_iter()
    .find(|u| api::text(u, "id") == id)
    .map(|u| api::text(&u, "username"))
    .unwrap_or_else(|| id.into())
}
#[component]
pub(super) fn IssueRow(issue: Value, base: String, users: Query) -> impl IntoView {
  let assigned = assignees(&issue)
    .into_iter()
    .map(|id| user_name(users, &id))
    .collect::<Vec<_>>()
    .join(", ");
  view! {
      <A href=format!("{base}/issues/{}", api::text(&issue, "number")) {..} class="issue-row">
          <div>
              <div>
                  <strong>"#"{api::text(&issue, "number")}" "{api::text(&issue, "title")}</strong>
                  <p>
                      "Opened by "{user_name(users, &api::text(&issue, "author_user_id"))}
                      " · updated "{api::text(&issue, "updated_at")}
                  </p>
              </div>
              <span class="badge" class:open=issue["status"] == "open">
                  {api::text(&issue, "status")}
              </span>
          </div>
          <p>{api::text(&issue, "description")}</p>
          <footer>
              <span class="badge outline">
                  "♙ Assignee: "
                  {if assigned.is_empty() { "Unassigned".into() } else { assigned }}
              </span>
              {if labels(&issue).is_empty() {
                  view! { <span class="badge outline">{crate::i18n::t("No labels")}</span> }
                      .into_any()
              } else {
                  labels(&issue)
                      .into_iter()
                      .map(|label| view! { <span class="badge">{label}</span> })
                      .collect_view()
                      .into_any()
              }}
          </footer>
      </A>
  }
}
