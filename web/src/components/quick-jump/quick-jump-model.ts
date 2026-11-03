import {
  BookOpen,
  Boxes,
  Code2,
  FolderGit2,
  GitBranch,
  GitCommit,
  GitPullRequest,
  Gauge,
  Home,
  Inbox,
  ListTodo,
  Package,
  PlayCircle,
  Rocket,
  Settings,
  Shield,
  User,
  Users,
  type LucideIcon,
} from "lucide-react";
import type { RepositoryView } from "@/pages/types";

export interface QuickJumpItem {
  id: string;
  title: string;
  description: string;
  path: string;
  icon: LucideIcon;
  keywords: string[];
}

export interface QuickJumpGroup {
  id: string;
  title: string;
  items: QuickJumpItem[];
}

export interface CurrentProjectRoute {
  organizationId: string;
  projectId: string;
}

type Translate = (value: string) => string;

const repositoryDestinations: Array<{
  tab: string;
  label: string;
  description: string;
  icon: LucideIcon;
  keywords: string[];
}> = [
  { tab: "overview", label: "Overview", description: "Project home, README, clone, and activity summary.", icon: Gauge, keywords: ["home", "summary"] },
  { tab: "code", label: "Code", description: "Browse files, README, languages, and code search.", icon: Code2, keywords: ["repository", "file", "search"] },
  { tab: "issues", label: "Issues", description: "Triage issues, assignees, labels, and comments.", icon: ListTodo, keywords: ["ticket", "label", "assignee"] },
  { tab: "merge-requests", label: "Merge Requests", description: "Review branches, approvals, checks, and blockers.", icon: GitPullRequest, keywords: ["mr", "review", "approval"] },
  { tab: "pipelines", label: "Pipelines", description: "Inspect CI pipeline state and job graph.", icon: Rocket, keywords: ["ci", "pipeline"] },
  { tab: "jobs", label: "Jobs", description: "Open job trace, artifacts, and diagnostics.", icon: PlayCircle, keywords: ["trace", "artifact", "log"] },
  { tab: "runners", label: "Runners", description: "Manage project runners, tags, and health.", icon: Users, keywords: ["runner", "agent"] },
  { tab: "packages", label: "Packages", description: "View registry packages and publish/install guidance.", icon: Package, keywords: ["registry", "maven", "npm"] },
  { tab: "wiki", label: "Wiki", description: "Open project documentation pages.", icon: BookOpen, keywords: ["docs", "documentation"] },
  { tab: "releases", label: "Releases", description: "View tags, release notes, and assets.", icon: Boxes, keywords: ["tag", "asset"] },
  { tab: "commits", label: "Commits", description: "Inspect commit history by branch.", icon: GitCommit, keywords: ["history"] },
  { tab: "branches", label: "Branches", description: "Manage branches and protection rules.", icon: GitBranch, keywords: ["protection"] },
  { tab: "settings", label: "Settings", description: "Project settings, credentials, members, and danger zone.", icon: Settings, keywords: ["credential", "member", "token"] },
];

export const buildQuickJumpGroups = (input: {
  canAccessAdmin: boolean;
  currentProjectRoute: CurrentProjectRoute | null;
  projects: RepositoryView[];
  t: Translate;
}): QuickJumpGroup[] => {
  const { canAccessAdmin, currentProjectRoute, projects, t } = input;
  const currentProject = currentProjectRoute
    ? projects.find((project) => project.id === currentProjectRoute.projectId) ?? null
    : null;
  const groups: QuickJumpGroup[] = [];

  if (currentProjectRoute) {
    const projectName = currentProject?.name ?? currentProjectRoute.projectId;
    const basePath = `/app/projects/${encodeURIComponent(currentProjectRoute.organizationId)}/${encodeURIComponent(currentProjectRoute.projectId)}`;
    groups.push({
      id: "current-project",
      title: t("Current Project"),
      items: repositoryDestinations.map((destination) => ({
        id: `repo-tab:${destination.tab}`,
        title: t(destination.label),
        description: t(destination.description),
        path: destination.tab === "overview" ? basePath : `${basePath}?tab=${encodeURIComponent(destination.tab)}`,
        icon: destination.icon,
        keywords: [
          destination.tab,
          destination.label,
          projectName,
          ...(currentProject ? [currentProject.key, currentProject.full_path] : []),
          ...destination.keywords,
        ],
      })),
    });
  }

  groups.push({
    id: "workspace",
    title: t("Workspace"),
    items: [
      quickItem("dashboard", t("Dashboard"), t("Open workspace activity and project shortcuts."), "/app/dashboard", Home, ["home", "workspace"]),
      quickItem("todos", t("Inbox"), t("Review pending project activity and completed todos."), "/app/todos", Inbox, ["todo", "notification", "pending"]),
      quickItem("projects", t("My Projects"), t("Browse organization projects and clone URLs."), "/app/projects", FolderGit2, ["repository", "repo"]),
      quickItem("profile", t("Profile"), t("Manage your current account profile."), "/app/profile", User, ["account", "user"]),
    ],
  });

  if (projects.length > 0) {
    groups.push({
      id: "projects",
      title: t("Projects"),
      items: projects.map((project) => ({
        id: `project:${project.id}`,
        title: project.name,
        description: project.full_path || project.description || t("Project workspace"),
        path: `/app/projects/${encodeURIComponent(project.organization_id)}/${encodeURIComponent(project.id)}`,
        icon: FolderGit2,
        keywords: [project.key, project.full_path, project.default_branch, project.visibility].filter(isNonEmptyString),
      })),
    });
  }

  if (canAccessAdmin) {
    groups.push({
      id: "administration",
      title: t("Administration"),
      items: [quickItem("admin", t("Admin Dashboard"), t("Review users, organizations, projects, and instance health."), "/admin", Shield, ["admin", "ops"])],
    });
  }

  return groups;
};

export const parseCurrentProjectRoute = (pathname: string): CurrentProjectRoute | null => {
  const match = /^\/app\/(?:projects|repositories)\/([^/]+)\/([^/]+)/.exec(pathname);
  if (!match) return null;
  return {
    organizationId: decodeURIComponent(match[1]),
    projectId: decodeURIComponent(match[2]),
  };
};

export const quickJumpValue = (item: QuickJumpItem): string => [item.id, item.title, item.description, item.path, ...item.keywords].join(" ");

const quickItem = (
  id: string,
  title: string,
  description: string,
  path: string,
  icon: LucideIcon,
  keywords: string[],
): QuickJumpItem => ({ id, title, description, path, icon, keywords });

const isNonEmptyString = (value: unknown): value is string => typeof value === "string" && value.length > 0;
