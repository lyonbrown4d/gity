use crate::{api, ui::*};
use leptos::prelude::*;
use leptos_router::{
  components::{Route, Router, Routes},
  hooks::use_location,
  path,
};

mod account;
mod admin_orgs;
mod admin_overview;
mod admin_projects;
mod admin_ui;
mod admin_users;
mod audit;
mod branches;
mod capabilities;
mod commits;
mod controls;
mod dashboard;
mod icons;
mod jobs;
mod lfs;
mod login;
mod merge_requests;
use controls::ViewControls;
mod mobile_navigation;
mod navigation;
mod overview;
mod packages;
mod pages;
mod pipelines;
mod profile;
mod project;
mod projects;
mod quick_jump;
mod releases;
mod runners;
mod settings;
mod theme;
mod todos;
mod wiki;
use login::Login;
use mobile_navigation::MobileNavigationDrawer;
use navigation::{AdminMobileNavigation, Navigation};
use pages::{AdminPage, Page};
use quick_jump::QuickJump;
#[cfg(target_arch = "wasm32")]
pub use theme::apply_theme;
use theme::ui_theme;
#[component]
pub fn App() -> impl IntoView {
  let theme = RwSignal::new(ui_theme());
  provide_context(theme);
  view! {
      <thaw::ConfigProvider theme=theme>
          <Router>
              <Routes fallback=|| view! { <p>{crate::i18n::t("Page not found")}</p> }>
                  <Route path=path!("/*any") view=Shell />
              </Routes>
          </Router>
      </thaw::ConfigProvider>
  }
}
#[component]
pub(super) fn Shell() -> impl IntoView {
  let location = use_location();
  let capabilities = query("/capabilities".into());
  provide_context(capabilities);
  let desktop_collapsed = RwSignal::new(false);
  let mobile_open = RwSignal::new(false);
  let route_path = location.pathname;
  let route_search = location.search;
  Effect::new(move |_| {
    route_path.get();
    route_search.get();
    mobile_open.set(false);
  });
  let me = query_if(
    "/users/me".into(),
    !api::storage("gity.access_token").is_empty(),
  );
  view! {
      {move || {
          let path = location.pathname.get();
          let search = location.search.get();
          if path == "/login" || api::storage("gity.access_token").is_empty() {
              return view! { <Login /> }.into_any();
          }
          let admin = path.starts_with("/admin");
          view! {
              <div class="app-shell" class:admin=admin class:collapsed=move || desktop_collapsed.get()>
                  <Navigation
                      path=path.clone()
                      search=search.clone()
                      me=me
                      collapsed=desktop_collapsed
                  />
                  <div class="workspace">
                      <header class="topbar" class:admin-topbar=admin>
                          <div class="topbar-breadcrumb">
                              {if admin {
                                  view! {
                                      <>
                                          <thaw::Icon
                                              icon=icondata_lu::LuShieldCheck
                                              width="16px"
                                              height="16px"
                                          />
                                          <span class="muted">"admin"</span>
                                          <span class="topbar-separator"></span>
                                          <strong>{route_title(&path)}</strong>
                                          <Show when=move || me.data.get()["is_super_admin"] == true>
                                              <span class="admin-super-badge">{crate::i18n::t("Super Admin")}</span>
                                          </Show>
                                      </>
                                  }
                                      .into_any()
                              } else {
                                  view! {
                                      <>
                                          <button
                                              aria-label=crate::i18n::t("Toggle Sidebar")
                                              aria-expanded=move || {
                                                  if is_mobile_viewport() {
                                                      mobile_open.get()
                                                  } else {
                                                      !desktop_collapsed.get()
                                                  }
                                              }
                                              on:click=move |_| {
                                                  if is_mobile_viewport() {
                                                      mobile_open.set(!mobile_open.get_untracked());
                                                  } else {
                                                      desktop_collapsed.set(
                                                          !desktop_collapsed.get_untracked(),
                                                      );
                                                  }
                                              }
                                          >
                                              <thaw::Icon
                                                  icon=icondata_lu::LuPanelLeft
                                                  width="16px"
                                                  height="16px"
                                              />
                                          </button>
                                          <span class="topbar-separator"></span>
                                  <span class="muted">{crate::i18n::t("User Workspace")}</span>
                                          <span class="muted">{crate::i18n::t("›")}</span>
                                          <strong>{route_title(&path)}</strong>
                                      </>
                                  }
                                      .into_any()
                              }}
                          </div>
                          <QuickJump />
                          <ViewControls />
                          <AdminMobileNavigation path=path.clone() />
                          <div class="admin-topbar-account">
                              <button
                                  class="admin-topbar-logout"
                                  aria-label=crate::i18n::t("Logout")
                                  title=crate::i18n::t("Logout")
                                  on:click=move |_| account::logout()
                              >
                                  <thaw::Icon
                                      icon=icondata_lu::LuLogOut
                                      width="14px"
                                      height="14px"
                                  />
                              </button>
                          </div>
                      </header>
                      <main id="main-content" class="main-content">
                          {if admin {
                              view! {
                                  <AdminPage path=path.clone() search=search.clone() me=me />
                              }
                                  .into_any()
                          } else {
                              view! { <Page path=path.clone() search=search.clone() /> }.into_any()
                          }}
                      </main>
                  </div>
                  <MobileNavigationDrawer
                      path=path.clone()
                      search=search.clone()
                      me=me
                      collapsed=desktop_collapsed
                      open=mobile_open
                      enabled=!admin
                  />
              </div>
          }
              .into_any()
      }}
  }
}

fn is_mobile_viewport() -> bool {
  web_sys::window()
    .and_then(|window| window.match_media("(max-width: 767px)").ok().flatten())
    .is_some_and(|query| query.matches())
}

fn route_title(path: &str) -> String {
  match path {
    "/app/dashboard" => crate::i18n::t("Dashboard"),
    "/app/todos" => crate::i18n::t("Inbox"),
    "/app/profile" => crate::i18n::t("Profile"),
    "/app/projects" => crate::i18n::t("My Projects"),
    "/admin/projects" => crate::i18n::t("Projects"),
    "/admin/orgs" => crate::i18n::t("Organizations"),
    "/admin/users" => crate::i18n::t("Users"),
    "/admin" => crate::i18n::t("Overview"),
    _ => crate::i18n::t("Project"),
  }
}
