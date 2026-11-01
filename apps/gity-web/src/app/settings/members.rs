mod table;
use super::{Header, Stat, model};
use crate::{api, i18n::t, ui::*};
use leptos::prelude::*;
use serde_json::json;

#[component]
pub(super) fn Members(project_path: String, permissions: Query) -> impl IntoView {
  let members = query(format!("{project_path}/members"));
  let users = query_all_pages("/users".into());
  let user = RwSignal::new(String::new());
  let role = RwSignal::new("developer".to_string());
  let search = RwSignal::new(String::new());
  let filter = RwSignal::new("all".to_string());
  let mutation = Mutation::new(members.refresh);
  let can_admin = Signal::derive(move || {
    allowed(
      &permissions.data.get(),
      "project.repository.admin",
      "repository_admin",
      40,
    )
  });
  let disabled = Signal::derive(move || !can_admin.get() || mutation.busy.get());
  let available = Memo::new(move |_| {
    api::list(&users.data.get())
      .into_iter()
      .filter(|u| {
        !api::list(&members.data.get())
          .iter()
          .any(|m| m["user_id"] == u["id"])
      })
      .collect::<Vec<_>>()
  });
  let submit_path = format!("{project_path}/members");
  let submit = move |e: leptos::ev::SubmitEvent| {
    e.prevent_default();
    if disabled.get_untracked() {
      return;
    }
    let Ok(id) = user.get_untracked().parse::<i64>() else {
      mutation
        .error
        .set(t("Select a user before adding a project member."));
      return;
    };
    mutation.run(
      "POST",
      submit_path.clone(),
      json!({"user_id":id,"role":role.get_untracked()}),
      Some(Callback::new(move |()| {
        user.set(String::new());
        role.set("developer".into());
      })),
    );
  };
  view! {
      <section class="settings-card account-card card-enter">
          <Header
              title="Project members"
              description="Project-specific roles override organization membership for this repository."
          />
          <div class="settings-body">
              <div class="settings-stats">
                  <Stat
                      label="Direct members"
                      value=Signal::derive(move || api::list(&members.data.get()).len())
                  />
                  <Stat
                      label="Maintainers/Owners"
                      value=Signal::derive(move || {
                          api::list(&members.data.get())
                              .iter()
                              .filter(|m| m["role"] == "maintainer" || m["role"] == "owner")
                              .count()
                      })
                  />
                  <Stat
                      label="Developers"
                      value=Signal::derive(move || {
                          api::list(&members.data.get())
                              .iter()
                              .filter(|m| m["role"] == "developer")
                              .count()
                      })
                  />
              </div>
              <Show when=move || !can_admin.get()>
                  <p class="settings-notice">
                      {t(
                          "Your current project role can inspect members, but cannot change project membership.",
                      )}
                  </p>
              </Show>
              <form class="settings-member-controls" on:submit=submit>
                  <label>
                      {t("User")}
                      <select
                          aria-label=t("User")
                          disabled=move || disabled.get() || available.get().is_empty()
                          prop:value=move || user.get()
                          on:change=move |e| user.set(event_target_value(&e))
                      >
                          <option value="">
                              {move || t(
                                  if available.get().is_empty() {
                                      "No users available"
                                  } else {
                                      "Select user"
                                  },
                              )}
                          </option>
                          {move || {
                              available
                                  .get()
                                  .into_iter()
                                  .map(|u| {
                                      view! {
                                          <option value=api::text(
                                              &u,
                                              "id",
                                          )>
                                              {format!(
                                                  "{} (@{})",
                                                  api::text(&u, "display_name"),
                                                  api::text(&u, "username"),
                                              )}
                                          </option>
                                      }
                                  })
                                  .collect_view()
                          }}
                      </select>
                  </label>
                  <label>
                      {t("Role")}
                      <RoleSelect
                          value=role.into()
                          disabled=disabled
                          on_change=Callback::new(move |v| role.set(v))
                      />
                  </label>
                  <button
                      type="submit"
                      class="primary"
                      disabled=move || disabled.get() || user.get().is_empty()
                  >
                      <thaw::Icon icon=icondata_lu::LuPlus width="16px" height="16px" />
                      {move || t(if mutation.busy.get() { "Adding..." } else { "Add member" })}
                  </button>
              </form>
              <div class="settings-member-controls">
                  <input
                      aria-label=t("Search members")
                      placeholder=t("Search members")
                      prop:value=move || search.get()
                      on:input=move |e| search.set(event_target_value(&e))
                  />
                  <select
                      aria-label=t("Role filter")
                      prop:value=move || filter.get()
                      on:change=move |e| filter.set(event_target_value(&e))
                  >
                      <option value="all">{t("All roles")}</option>
                      {model::ROLES
                          .into_iter()
                          .map(|role| view! { <option value=role>{t(role)}</option> })
                          .collect_view()}
                  </select>
                  <button
                      type="button"
                      on:click=move |_| {
                          members.refresh.update(|v| *v += 1);
                          users.refresh.update(|v| *v += 1);
                      }
                  >
                      {t("Reload")}
                  </button>
              </div>
              <Status q=members />
              <Status q=users />
              <Show when=move || !mutation.error.get().is_empty()>
                  <p role="alert" class="error">
                      {move || mutation.error.get()}
                  </p>
              </Show>
              <table::MemberTable
                  project_path=project_path
                  members=members
                  users=users
                  mutation=mutation
                  disabled=disabled
                  search=search
                  filter=filter
              />
          </div>
      </section>
  }
}
#[component]
fn MemberRole(
  initial: String,
  disabled: Signal<bool>,
  on_change: Callback<String>,
) -> impl IntoView {
  let previous = initial.clone();
  let value = RwSignal::new(initial);
  view! {
      <RoleSelect
          value=value.into()
          disabled=disabled
          on_change=Callback::new(move |role: String| {
              value.set(previous.clone());
              on_change.run(role);
          })
      />
  }
}
#[component]
fn RoleSelect(
  value: Signal<String>,
  disabled: Signal<bool>,
  on_change: Callback<String>,
) -> impl IntoView {
  view! {
      <select
          aria-label=t("Change role")
          prop:value=move || value.get()
          disabled=move || disabled.get()
          on:change=move |e| on_change.run(event_target_value(&e))
      >
          {model::ROLES
              .into_iter()
              .map(|role| view! { <option value=role>{t(role)}</option> })
              .collect_view()}
      </select>
  }
}
