import {
  Activity, Boxes, Building2, FolderGit2, Gauge, GitBranch, Inbox, ShieldCheck, User, Users,
} from "lucide-react";
import {
  SidebarContent, SidebarFooter, SidebarHeader,
} from "@/components/ui/sidebar";
import type { RepositoryView } from "@/pages/types";
import { ContextNavigationGroup } from "./context-navigation-group";
import { ProjectContextNavigation } from "./project-context-navigation";
import type { NavigationArea, ProjectRoute } from "./navigation-model";

interface ContextNavigationPanelProps {
  activeArea: NavigationArea;
  currentTab: string;
  projectRoute: ProjectRoute | null;
  projects: RepositoryView[];
  todoState: "pending" | "done";
  user: { name: string; email: string };
  onNavigate: () => void;
  t: (value: string) => string;
}

export function ContextNavigationPanel({
  activeArea,
  currentTab,
  projectRoute,
  projects,
  todoState,
  user,
  onNavigate,
  t,
}: ContextNavigationPanelProps): JSX.Element {
  const currentProject = projectRoute
    ? projects.find((project) => project.id === projectRoute.projectId) ?? null
    : null;
  const heading = contextHeading(activeArea, currentProject, t);

  return (
    <div className="flex min-w-0 flex-1 flex-col bg-muted text-foreground group-data-[collapsible=icon]:hidden">
      <SidebarHeader className="h-16 justify-center gap-0.5 border-b border-border px-4 py-2">
        <p className="font-mono text-[9px] font-semibold uppercase tracking-[0.2em] text-primary">{heading.eyebrow}</p>
        <p className="truncate text-sm font-semibold tracking-tight">{heading.title}</p>
        <p className="truncate text-[11px] text-muted-foreground">{heading.description}</p>
      </SidebarHeader>

      <SidebarContent className="gap-0 px-1 py-2">
        {activeArea === "workspace" ? <WorkspaceNavigation onNavigate={onNavigate} t={t} /> : null}
        {activeArea === "projects" && !projectRoute ? (
          <ProjectsNavigation projects={projects} onNavigate={onNavigate} t={t} />
        ) : null}
        {activeArea === "projects" && projectRoute ? (
          <ProjectContextNavigation route={projectRoute} currentTab={currentTab} onNavigate={onNavigate} t={t} />
        ) : null}
        {activeArea === "inbox" ? (
          <InboxNavigation state={todoState} onNavigate={onNavigate} t={t} />
        ) : null}
        {activeArea === "admin" ? <AdminNavigation onNavigate={onNavigate} t={t} /> : null}
      </SidebarContent>

      <SidebarFooter className="border-t border-border px-4 py-3">
        <p className="truncate text-xs font-medium">{user.name}</p>
        <p className="truncate text-[10px] text-muted-foreground">{user.email}</p>
      </SidebarFooter>
    </div>
  );
}

function WorkspaceNavigation({ onNavigate, t }: NavigationProps): JSX.Element {
  return <ContextNavigationGroup label={t("Workspace")} items={[
    { title: t("Dashboard"), url: "/app/dashboard", icon: Gauge },
    { title: t("Activity"), url: "/app/activity", icon: Activity },
    { title: t("Profile"), url: "/app/profile", icon: User },
  ]} onNavigate={onNavigate} />;
}

function ProjectsNavigation({ projects, onNavigate, t }: NavigationProps & { projects: RepositoryView[] }): JSX.Element {
  return (
    <>
      <ContextNavigationGroup label={t("Projects")} items={[
        { title: t("All projects"), url: "/app/projects", icon: FolderGit2 },
      ]} onNavigate={onNavigate} />
      {projects.length > 0 ? (
        <ContextNavigationGroup
          label={t("Recent")}
          items={projects.slice(0, 6).map((project) => ({
            title: project.name,
            url: `/app/projects/${encodeURIComponent(project.organization_id)}/${encodeURIComponent(project.id)}`,
            icon: GitBranch,
          }))}
          onNavigate={onNavigate}
        />
      ) : null}
    </>
  );
}

function InboxNavigation({ state, onNavigate, t }: NavigationProps & { state: "pending" | "done" }): JSX.Element {
  return <ContextNavigationGroup label={t("Queues")} items={[
    { title: t("Pending"), url: "/app/todos?state=pending", icon: Inbox },
    { title: t("Done"), url: "/app/todos?state=done", icon: ShieldCheck },
  ]} activeUrl={`/app/todos?state=${state}`} onNavigate={onNavigate} />;
}

function AdminNavigation({ onNavigate, t }: NavigationProps): JSX.Element {
  return <ContextNavigationGroup label={t("Administration")} items={[
    { title: t("Overview"), url: "/admin", icon: Gauge },
    { title: t("Organizations"), url: "/admin/orgs", icon: Building2 },
    { title: t("Projects"), url: "/admin/projects", icon: Boxes },
    { title: t("Users"), url: "/admin/users", icon: Users },
  ]} onNavigate={onNavigate} />;
}

interface NavigationProps {
  onNavigate: () => void;
  t: (value: string) => string;
}

function contextHeading(area: NavigationArea, project: RepositoryView | null, t: (value: string) => string) {
  if (area === "projects" && project) {
    return { eyebrow: t("Project"), title: project.name, description: project.full_path };
  }
  const headings = {
    workspace: { eyebrow: t("Personal space"), title: t("Workspace"), description: t("Your command center") },
    projects: { eyebrow: t("Code space"), title: t("Projects"), description: t("Code and collaboration") },
    inbox: { eyebrow: t("Review queue"), title: t("Inbox"), description: t("Work requiring attention") },
    admin: { eyebrow: t("Control plane"), title: t("Administration"), description: t("Platform operations") },
  };
  return headings[area];
}
