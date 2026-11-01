use super::{MemberRole, model};
use crate::{api, i18n::t, ui::*};
use leptos::prelude::*;
use serde_json::{Value, json};
#[component]
pub(super) fn MemberTable(
  project_path: String,
  members: Query,
  users: Query,
  mutation: Mutation,
  disabled: Signal<bool>,
  search: RwSignal<String>,
  filter: RwSignal<String>,
) -> impl IntoView {
  let enriched = Memo::new(move |_| {
    Value::Array(
      api::list(&members.data.get())
        .into_iter()
        .map(|mut member| {
          if let Some(user) = api::list(&users.data.get())
            .into_iter()
            .find(|user| user["id"] == member["user_id"])
          {
            member["email"] = user["email"].clone();
          }
          member
        })
        .collect(),
    )
  });
  view! {
      <div class="settings-table-scroll">
          <table>
              <thead>
                  <tr>
                      {["Member", "Source", "Role", "Change role", "Actions"]
                          .into_iter()
                          .map(|h| view! { <th>{t(h)}</th> })
                          .collect_view()}
                  </tr>
              </thead>
              <tbody>
                  {move || {
                      model::filtered(&enriched.get(), &search.get(), &filter.get())
                          .into_iter()
                          .map(|member| {
                              let member_id = api::text(&member, "user_id");
                              let path = format!(
                                  "{project_path}/members/{}",
                                  api::enc(&member_id),
                              );
                              let change_path = path.clone();
                              let initial = api::text(&member, "role");
                              let email = api::list(&users.data.get())
                                  .into_iter()
                                  .find(|u| api::text(u, "id") == member_id)
                                  .map(|u| api::text(&u, "email"))
                                  .filter(|v| !v.is_empty())
                                  .unwrap_or_else(|| t("No email"));
                              view! {
                                  <tr>
                                      <td>
                                          <strong>{api::text(&member, "display_name")}</strong>
                                          <small>
                                              {format!(
                                                  "@{} · {}",
                                                  api::text(&member, "username"),
                                                  email,
                                              )}
                                          </small>
                                      </td>
                                      <td>
                                          <span class="badge outline">"project"</span>
                                      </td>
                                      <td>
                                          <span class="badge">{t(&initial)}</span>
                                          <small>{t(model::role_description(&initial))}</small>
                                      </td>
                                      <td>
                                          <MemberRole
                                              initial=initial
                                              disabled=disabled
                                              on_change=Callback::new(move |value| {
                                                  mutation
                                                      .run(
                                                          "PATCH",
                                                          change_path.clone(),
                                                          json!({"role":value}),
                                                          None,
                                                      )
                                              })
                                          />
                                      </td>
                                      <td>
                                          <Action
                                              label="Remove".into()
                                              path=path
                                              method="DELETE".into()
                                              refresh=members.refresh
                                              disabled=disabled
                                              busy_state=mutation.busy
                                              confirm=true
                                              confirmation_title=t("Remove project member?")
                                              confirmation_description=t(
                                                  "This removes only the project-specific role. Organization membership is not changed.",
                                              )
                                              confirmation_label=t("Remove")
                                          >
                                              <thaw::Icon icon=icondata_lu::LuTrash2 width="14px" height="14px" />
                                              {t("Remove")}
                                          </Action>
                                      </td>
                                  </tr>
                              }
                          })
                          .collect_view()
                  }}
                  <Show when=move || {
                      !members.loading.get()
                          && model::filtered(&enriched.get(), &search.get(), &filter.get())
                              .is_empty()
                  }>
                      <tr>
                          <td colspan="5">
                              {move || t(
                                  if api::list(&members.data.get()).is_empty() {
                                      "No project-specific members yet."
                                  } else {
                                      "No members match the current filters."
                                  },
                              )}
                          </td>
                      </tr>
                  </Show>
              </tbody>
          </table>
      </div>
  }
}
