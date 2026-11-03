import {
  Activity, Boxes, Building2, FolderGit2, Gauge, GitBranch, GitPullRequest, Inbox, ListTodo,
  Package, PlayCircle, Rocket, ScrollText, Settings, ShieldCheck, User, Users,
} from "lucide-react";
import { Link } from "react-router";
import { NavUser } from "@/components/nav-user";
import { SidebarContent, SidebarFooter, SidebarHeader } from "@/components/ui/sidebar";
import type { RepositoryView } from "@/pages/types";
import { ContextNavigationGroup } from "./context-navigation-group";
import { projectBasePath, type NavigationArea, type ProjectRoute } from "./navigation-model";
import type { ContextNavigationItem } from "./context-navigation-group";

interface ContextNavigationPanelProps {
  activeArea: NavigationArea;
  currentTab: string;
  projectRoute: ProjectRoute | null;
  projects: RepositoryView[];
  todoState: "pending" | "done";
  user: { name: string; email: string; isSuperAdmin: boolean };
  onLogout: () => void;
  onNavigate: () => void;
  t: (value: string) => string;
}

export const ContextNavigationPanel = ({
  activeArea, currentTab, projectRoute, projects, todoState, user, onLogout, onNavigate, t,
}: ContextNavigationPanelProps): JSX.Element => {
  const currentProject = projectRoute ? projects.find((project) => project.id === projectRoute.projectId) ?? null : null;
  const heading = contextHeading(activeArea, currentProject, t);

  return (
    <div className="flex min-h-0 w-full flex-1 flex-col bg-sidebar text-sidebar-foreground">
      <SidebarHeader className="h-14 flex-row items-center justify-start gap-2 border-b border-sidebar-border px-3 py-0">
        <Link to="/app/dashboard" onClick={onNavigate} aria-label={t("Gity home")} className="flex min-w-0 flex-1 items-center gap-2 rounded-sm outline-none focus-visible:ring-2 focus-visible:ring-sidebar-ring">
          <span className="grid size-8 shrink-0 place-items-center rounded-md bg-sidebar-primary text-sm font-bold text-sidebar-primary-foreground">G</span>
          <span className="truncate text-base font-semibold tracking-tight group-data-[collapsible=icon]:hidden">Gity</span>
        </Link>
        <span className="truncate text-xs text-sidebar-foreground/65 group-data-[collapsible=icon]:hidden">{heading.title}</span>
      </SidebarHeader>

      <SidebarContent className="gap-0 py-2">
        <nav aria-label={t("Primary navigation")} className="space-y-0.5">
          {activeArea === "workspace" ? <WorkspaceNavigation isSuperAdmin={user.isSuperAdmin} onNavigate={onNavigate} t={t} /> : null}
          {activeArea === "projects" && !projectRoute ? <ProjectsNavigation projects={projects} onNavigate={onNavigate} t={t} /> : null}
          {activeArea === "projects" && projectRoute ? <ProjectNavigation route={projectRoute} currentTab={currentTab} onNavigate={onNavigate} t={t} /> : null}
          {activeArea === "inbox" ? <InboxNavigation state={todoState} onNavigate={onNavigate} t={t} /> : null}
          {activeArea === "admin" ? <AdminNavigation onNavigate={onNavigate} t={t} /> : null}
        </nav>
      </SidebarContent>

      <SidebarFooter className="border-t border-sidebar-border p-2">
        <NavUser user={user} onLogout={onLogout} onNavigate={onNavigate} compact />
      </SidebarFooter>
    </div>
  );
};

interface NavigationProps {
  onNavigate: () => void;
  t: (value: string) => string;
}

const WorkspaceNavigation = ({ isSuperAdmin, onNavigate, t }: NavigationProps & { isSuperAdmin: boolean }): JSX.Element => (
    <ContextNavigationGroup label={t("Workspace")} collapsible items={[
    { title: t("Dashboard"), url: "/app/dashboard", icon: Gauge },
    { title: t("Projects"), url: "/app/projects", icon: FolderGit2 },
    { title: t("Inbox"), url: "/app/todos", icon: Inbox },
    { title: t("Activity"), url: "/app/activity", icon: Activity },
    { title: t("Profile"), url: "/app/profile", icon: User },
    ...(isSuperAdmin ? [{ title: t("Admin Dashboard"), url: "/admin", icon: ShieldCheck }] : []),
  ]} onNavigate={onNavigate} />
);

const ProjectsNavigation = ({ projects, onNavigate, t }: NavigationProps & { projects: RepositoryView[] }): JSX.Element => (
  <>
    <ContextNavigationGroup label={t("Projects")} collapsible items={[
      { title: t("All projects"), url: "/app/projects", icon: FolderGit2 },
    ]} onNavigate={onNavigate} />
    {projects.length ? <ContextNavigationGroup label={t("Recent projects")} collapsible items={projects.slice(0, 6).map((project) => ({
      title: project.name,
      url: `/app/projects/${encodeURIComponent(project.organization_id)}/${encodeURIComponent(project.id)}`,
      icon: GitBranch,
    }))} onNavigate={onNavigate} /> : null}
  </>
);

const projectGroups: Array<{ label: string; items: Array<{ tab: string; title: string; icon: ContextNavigationItem["icon"] }> }> = [
  { label: "Project", items: [
    { tab: "overview", title: "Overview", icon: Activity }, { tab: "code", title: "Code", icon: GitBranch },
    { tab: "wiki", title: "Wiki", icon: ScrollText }, { tab: "packages", title: "Packages", icon: Package },
    { tab: "releases", title: "Releases", icon: Boxes },
  ] },
  { label: "Plan", items: [
    { tab: "issues", title: "Issues", icon: ListTodo }, { tab: "merge-requests", title: "Merge Requests", icon: GitPullRequest },
  ] },
  { label: "Repository", items: [
    { tab: "commits", title: "Commits", icon: GitBranch }, { tab: "branches", title: "Branches", icon: GitBranch },
    { tab: "lfs", title: "LFS", icon: Package },
  ] },
  { label: "CI/CD", items: [
    { tab: "pipelines", title: "Pipelines", icon: Rocket }, { tab: "pipelines/editor", title: "Pipeline editor", icon: PlayCircle },
    { tab: "jobs", title: "Jobs", icon: PlayCircle },
    { tab: "runners", title: "Runners", icon: Users },
  ] },
  { label: "Operate", items: [
    { tab: "audit", title: "Audit", icon: ScrollText }, { tab: "settings", title: "Settings", icon: Settings },
  ] },
];

const ProjectNavigation = ({ route, currentTab, onNavigate, t }: NavigationProps & { route: ProjectRoute; currentTab: string }): JSX.Element => {
  const basePath = projectBasePath(route);
  const selectedTab = currentTab === "pipelines/editor" ? currentTab : currentTab.split("/")[0];
  const activeUrl = selectedTab === "overview" ? basePath : `${basePath}/${selectedTab}`;
  return <>
    <ContextNavigationGroup label={t("Project")} items={[{ title: t("Back to projects"), url: "/app/projects", icon: FolderGit2 }]} activeUrl="none" onNavigate={onNavigate} />
    {projectGroups.map((group) => <ContextNavigationGroup key={group.label} label={t(group.label)} collapsible items={group.items.map((item) => ({
      title: t(item.title), url: item.tab === "overview" ? basePath : `${basePath}/${item.tab}`, icon: item.icon,
    }))} activeUrl={activeUrl} onNavigate={onNavigate} />)}
  </>;
};

const InboxNavigation = ({ state, onNavigate, t }: NavigationProps & { state: "pending" | "done" }): JSX.Element => (
  <ContextNavigationGroup label={t("Queues")} collapsible items={[
    { title: t("Pending"), url: "/app/todos?state=pending", icon: Inbox },
    { title: t("Done"), url: "/app/todos?state=done", icon: ShieldCheck },
  ]} activeUrl={`/app/todos?state=${state}`} onNavigate={onNavigate} />
);

const AdminNavigation = ({ onNavigate, t }: NavigationProps): JSX.Element => (
  <ContextNavigationGroup label={t("Administration")} collapsible items={[
    { title: t("Overview"), url: "/admin", icon: Gauge }, { title: t("Organizations"), url: "/admin/orgs", icon: Building2 },
    { title: t("Projects"), url: "/admin/projects", icon: Boxes }, { title: t("Users"), url: "/admin/users", icon: Users },
  ]} onNavigate={onNavigate} />
);

const contextHeading = (area: NavigationArea, project: RepositoryView | null, t: (value: string) => string) => {
  if (area === "projects" && project) return { title: project.name };
  const titles: Record<NavigationArea, string> = {
    workspace: t("Workspace"), projects: t("Projects"), inbox: t("Inbox"), admin: t("Administration"),
  };
  return { title: titles[area] };
};
