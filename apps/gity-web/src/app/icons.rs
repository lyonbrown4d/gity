use leptos::prelude::*;
#[component]
pub(super) fn NavigationIcon(title: String, #[prop(default = false)] admin: bool) -> impl IntoView {
  let icon = match title.as_str() {
    "Dashboard" => icondata_lu::LuGauge,
    "Profile" => icondata_lu::LuUser,
    "All projects" => icondata_lu::LuFolderGit2,
    "Back to projects" => icondata_lu::LuArrowLeft,
    "Overview" if admin => icondata_lu::LuLayoutDashboard,
    "Overview" => icondata_lu::LuActivity,
    "Organizations" => icondata_lu::LuBuilding2,
    "Projects" => icondata_lu::LuBoxes,
    "Users" => icondata_lu::LuUsersRound,
    "Code" => icondata_lu::LuCodeXml,
    "Wiki" => icondata_lu::LuBookOpen,
    "Packages" => icondata_lu::LuPackage,
    "Releases" => icondata_lu::LuBoxes,
    "Issues" => icondata_lu::LuListTodo,
    "Merge Requests" => icondata_lu::LuGitPullRequest,
    "Commits" => icondata_lu::LuGitCommitHorizontal,
    "Branches" => icondata_lu::LuGitBranch,
    "LFS" => icondata_lu::LuDatabase,
    "Pipelines" => icondata_lu::LuRocket,
    "Jobs" => icondata_lu::LuCirclePlay,
    "Runners" => icondata_lu::LuUsers,
    "Audit" => icondata_lu::LuScrollText,
    "Settings" => icondata_lu::LuSettings,
    "Pending" => icondata_lu::LuInbox,
    "Done" => icondata_lu::LuShieldCheck,
    _ => icondata_lu::LuGitBranch,
  };
  view! {
      <span class="context-icon">
          <thaw::Icon icon=icon width="14px" height="14px" />
      </span>
  }
}
