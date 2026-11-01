mod directory;
mod forms;
mod model;
use super::admin_ui::{
  ConsoleHeader, ConsoleMetric, ConsoleMetrics, ConsolePagination, ConsolePanel,
};
use crate::{api, i18n::t, ui::*};
use leptos::prelude::*;
use model::user_page;
use serde_json::Value;

#[component]
pub(super) fn AdminUsers() -> impl IntoView {
  let page = RwSignal::new(1u64);
  let size = RwSignal::new(20u64);
  let q = query_fn(
    move || format!("/users?page={}&page_size={}", page.get(), size.get()),
    true,
  );
  let parsed = Memo::new(move |_| user_page(&q.data.get()));
  let rows = Memo::new(move |_| parsed.get().unwrap_or_default());
  Effect::new(move |_| {
    if !q.loading.get()
      && q.error.get().is_empty()
      && let Err(error) = parsed.get()
    {
      q.error.set(t(error));
    }
  });
  let create_open = RwSignal::new(false);
  let edit_open = RwSignal::new(false);
  let selected = RwSignal::new(Value::Null);
  let creation = Mutation::new(q.refresh);
  let update = Mutation::new(q.refresh);
  let active_count = Signal::derive(move || {
    rows
      .get()
      .rows
      .iter()
      .filter(|user| api::text(user, "status").eq_ignore_ascii_case("active"))
      .count()
  });
  let admin_count = Signal::derive(move || {
    rows
      .get()
      .rows
      .iter()
      .filter(|user| user["is_super_admin"] == true)
      .count()
  });
  let on_edit = Callback::new(move |user| {
    selected.set(user);
    update.error.set(String::new());
    edit_open.set(true);
  });
  view! {
      <div class="admin-users console-page page-enter">
          <ConsoleHeader title="Users" description="User accounts in current admin scope.">
              <button
                  class="primary"
                  type="button"
                  on:click=move |_| {
                      creation.error.set(String::new());
                      create_open.set(true);
                  }
              >
                  <thaw::Icon icon=icondata_lu::LuPlus width="14px" height="14px" />
                  {t("New User")}
              </button>
          </ConsoleHeader>
          <Show when=move || !q.error.get().is_empty()>
              <p class="error" role="alert">
                  {move || q.error.get()}
              </p>
          </Show>
          <ConsoleMetrics items=vec![
              ConsoleMetric {
                  label: "Users",
                  value: Signal::derive(move || rows.get().total.to_string()),
                  detail: t("total"),
                  active: Signal::derive(move || rows.get().total > 0),
              },
              ConsoleMetric {
                  label: "Status",
                  value: Signal::derive(move || active_count.get().to_string()),
                  detail: format!("{} (page)", t("active")),
                  active: Signal::derive(move || active_count.get() > 0),
              },
              ConsoleMetric {
                  label: "Privilege level",
                  value: Signal::derive(move || admin_count.get().to_string()),
                  detail: format!("{} (page)", t("super-admin")),
                  active: Signal::derive(move || admin_count.get() > 0),
              },
          ] />
          <ConsolePanel
              title="User Directory"
              description="Identity, status, and privilege level."
              meta=Signal::derive(move || format!("{} {}", rows.get().total, t("total")))
          >
              <directory::Directory
                  rows=rows
                  loading=q.loading
                  refresh=q.refresh
                  on_edit=on_edit
              />
              <ConsolePagination
                  page=page
                  page_size=size
                  total=Signal::derive(move || rows.get().total as usize)
                  id_prefix="users"
              />
          </ConsolePanel>
          <thaw::Dialog open=create_open>
              <thaw::DialogSurface class="user-dialog">
                  <thaw::DialogBody>
                      <thaw::DialogTitle>{t("Create User")}</thaw::DialogTitle>
                      <thaw::DialogContent>
                          <Show when=move || create_open.get()>
                              <forms::UserForm
                                  initial=Value::Null
                                  mutation=creation
                                  on_cancel=Callback::new(move |_| create_open.set(false))
                                  on_saved=Callback::new(move |_| create_open.set(false))
                              />
                          </Show>
                      </thaw::DialogContent>
                  </thaw::DialogBody>
              </thaw::DialogSurface>
          </thaw::Dialog>
          <thaw::Dialog open=edit_open>
              <thaw::DialogSurface class="user-dialog">
                  <thaw::DialogBody>
                      <thaw::DialogTitle>{t("Edit User")}</thaw::DialogTitle>
                      <thaw::DialogContent>
                          <Show when=move || edit_open.get()>
                              <forms::UserForm
                                  initial=selected.get()
                                  mutation=update
                                  on_cancel=Callback::new(move |_| edit_open.set(false))
                                  on_saved=Callback::new(move |_| edit_open.set(false))
                              />
                          </Show>
                      </thaw::DialogContent>
                  </thaw::DialogBody>
              </thaw::DialogSurface>
          </thaw::Dialog>
      </div>
  }
}
