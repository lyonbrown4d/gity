import { useEffect, useMemo, useState } from "react";
import { useList } from "@refinedev/core";
import { useLocation, useNavigate } from "react-router-dom";
import {
  BookOpen,
  Boxes,
  Briefcase,
  Code2,
  FileSearch,
  FolderGit2,
  GitBranch,
  GitCommit,
  GitPullRequest,
  Gauge,
  Home,
  ListTodo,
  Package,
  PlayCircle,
  Rocket,
  Search,
  Settings,
  Shield,
  User,
  Users,
  type LucideIcon,
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useI18n } from "@/lib/i18n";
import type { RepositoryView } from "@/pages/types";

interface QuickJumpItem {
  id: string;
  title: string;
  description: string;
  section: string;
  path: string;
  icon: LucideIcon;
  keywords: string[];
}

interface CurrentProjectRoute {
  organizationId: string;
  projectId: string;
}

const repositoryTabs: Array<{ tab: string; label: string; description: string; icon: LucideIcon; keywords: string[] }> = [
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

export function GlobalQuickJump(): JSX.Element {
  const { t } = useI18n();
  const navigate = useNavigate();
  const location = useLocation();
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const projectQuery = useList<RepositoryView>({
    resource: "my-projects",
    pagination: { currentPage: 1, pageSize: 30 },
    meta: { all: true },
    queryOptions: {
      refetchOnWindowFocus: false,
      retry: false,
    },
  });
  const projects = projectQuery.result.data ?? [];
  const currentProjectRoute = useMemo(() => parseCurrentProjectRoute(location.pathname), [location.pathname]);
  const currentProject = currentProjectRoute
    ? projects.find((project) => project.id === currentProjectRoute.projectId) ?? null
    : null;

  const items = useMemo(() => {
    const globalItems: QuickJumpItem[] = [
      {
        id: "dashboard",
        title: t("Dashboard"),
        description: t("Open workspace activity and project shortcuts."),
        section: t("Workspace"),
        path: "/app/dashboard",
        icon: Home,
        keywords: ["home", "workspace", "dashboard"],
      },
      {
        id: "projects",
        title: t("My Projects"),
        description: t("Browse organization projects and clone URLs."),
        section: t("Workspace"),
        path: "/app/projects",
        icon: FolderGit2,
        keywords: ["repository", "repo", "project"],
      },
      {
        id: "profile",
        title: t("Profile"),
        description: t("Manage your current account profile."),
        section: t("Workspace"),
        path: "/app/profile",
        icon: User,
        keywords: ["account", "user"],
      },
      {
        id: "admin",
        title: t("Admin Dashboard"),
        description: t("Review users, organizations, projects, and instance health."),
        section: t("Admin"),
        path: "/admin",
        icon: Shield,
        keywords: ["admin", "ops"],
      },
    ];

    const projectItems = projects.map((project): QuickJumpItem => ({
      id: `project:${project.id}`,
      title: project.name,
      description: project.full_path || project.description || t("Project workspace"),
      section: t("Projects"),
      path: `/app/projects/${encodeURIComponent(project.organization_id)}/${encodeURIComponent(project.id)}`,
      icon: FolderGit2,
      keywords: [project.key, project.full_path, project.default_branch, project.visibility].filter((value): value is string => typeof value === "string" && value.length > 0),
    }));

    const routeItems = currentProjectRoute
      ? repositoryTabs.map((tab): QuickJumpItem => {
        const projectName = currentProject?.name ?? currentProjectRoute.projectId;
        const basePath = `/app/projects/${encodeURIComponent(currentProjectRoute.organizationId)}/${encodeURIComponent(currentProjectRoute.projectId)}`;
        return {
          id: `repo-tab:${tab.tab}`,
          title: `${projectName}: ${t(tab.label)}`,
          description: t(tab.description),
          section: t("Current Project"),
          path: tab.tab === "overview" ? basePath : `${basePath}?tab=${encodeURIComponent(tab.tab)}`,
          icon: tab.icon,
          keywords: [tab.tab, tab.label, projectName, ...(currentProject ? [currentProject.key, currentProject.full_path] : []), ...tab.keywords],
        };
      })
      : [];

    return [...routeItems, ...globalItems, ...projectItems];
  }, [currentProject, currentProjectRoute, projects, t]);

  const filteredItems = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase();
    if (!normalizedQuery) {
      return items.slice(0, 18);
    }
    return items
      .map((item) => ({ item, score: scoreItem(item, normalizedQuery) }))
      .filter((entry) => entry.score > 0)
      .sort((left, right) => right.score - left.score || left.item.title.localeCompare(right.item.title))
      .slice(0, 24)
      .map((entry) => entry.item);
  }, [items, query]);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setOpen((current) => !current);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  useEffect(() => {
    if (!open) {
      setQuery("");
    }
  }, [open]);

  const openItem = (item: QuickJumpItem) => {
    navigate(item.path);
    setOpen(false);
  };

  return (
    <>
      <Button type="button" variant="outline" size="sm" className="hidden gap-2 md:inline-flex" onClick={() => setOpen(true)}>
        <Search className="size-4" />
        {t("Quick jump")}
        <kbd className="rounded border bg-muted px-1.5 py-0.5 text-[10px] font-semibold text-muted-foreground">Ctrl K</kbd>
      </Button>
      <Button type="button" variant="ghost" size="icon" className="md:hidden" aria-label={t("Quick jump")} onClick={() => setOpen(true)}>
        <Search className="size-4" />
      </Button>
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent className="max-w-2xl gap-3 p-0">
          <DialogHeader className="border-b px-5 pb-3 pt-5">
            <DialogTitle className="flex items-center gap-2">
              <FileSearch className="size-5" />
              {t("Quick jump")}
            </DialogTitle>
            <DialogDescription>{t("Search projects, repository sections, and workspace destinations.")}</DialogDescription>
          </DialogHeader>
          <div className="px-5">
            <Input
              autoFocus
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder={t("Search projects, issues, merge requests, files...")}
            />
          </div>
          <ScrollArea className="max-h-[56vh] px-5 pb-5">
            <div className="flex flex-col gap-2 pt-2">
              {filteredItems.map((item) => (
                <button
                  key={item.id}
                  type="button"
                  className="flex w-full items-center gap-3 rounded-xl border border-border/70 bg-card/70 p-3 text-left transition hover:border-primary/40 hover:bg-accent"
                  onClick={() => openItem(item)}
                >
                  <span className="flex size-10 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary">
                    <item.icon className="size-4" />
                  </span>
                  <span className="min-w-0 flex-1">
                    <span className="block truncate text-sm font-semibold">{item.title}</span>
                    <span className="block truncate text-xs text-muted-foreground">{item.description}</span>
                  </span>
                  <Badge variant="secondary" className="shrink-0">{item.section}</Badge>
                </button>
              ))}
              {filteredItems.length === 0 ? (
                <div className="rounded-xl border border-dashed p-5 text-sm text-muted-foreground">
                  {projectQuery.query.isError
                    ? t("Project shortcuts could not be loaded, but global navigation is still available.")
                    : t("No quick jump results.")}
                </div>
              ) : null}
            </div>
          </ScrollArea>
        </DialogContent>
      </Dialog>
    </>
  );
}

const parseCurrentProjectRoute = (pathname: string): CurrentProjectRoute | null => {
  const match = /^\/app\/(?:projects|repositories)\/([^/]+)\/([^/]+)/.exec(pathname);
  if (!match) {
    return null;
  }
  return {
    organizationId: decodeURIComponent(match[1]),
    projectId: decodeURIComponent(match[2]),
  };
};

const scoreItem = (item: QuickJumpItem, query: string): number => {
  const haystack = [item.title, item.description, item.section, item.path, ...item.keywords]
    .join(" ")
    .toLowerCase();
  if (item.title.toLowerCase().startsWith(query)) {
    return 4;
  }
  if (haystack.includes(query)) {
    return 2;
  }
  return query.split(/\s+/).every((part) => haystack.includes(part)) ? 1 : 0;
};

