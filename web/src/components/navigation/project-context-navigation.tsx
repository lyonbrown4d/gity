import {
  Activity, ArrowLeft, BookOpen, Boxes, Code2, Database, GitBranch, GitCommit,
  GitPullRequest, ListTodo, Package, PlayCircle, Rocket, ScrollText, Settings, Users,
  type LucideIcon,
} from "lucide-react";
import { ContextNavigationGroup } from "./context-navigation-group";
import { projectBasePath, type ProjectRoute } from "./navigation-model";

interface ProjectContextNavigationProps {
  route: ProjectRoute;
  currentTab: string;
  onNavigate: () => void;
  t: (value: string) => string;
}

const projectGroups: Array<{
  label: string;
  items: Array<{ tab: string; title: string; icon: LucideIcon }>;
}> = [
  { label: "Project", items: [
    { tab: "overview", title: "Overview", icon: Activity },
    { tab: "code", title: "Code", icon: Code2 },
    { tab: "wiki", title: "Wiki", icon: BookOpen },
    { tab: "packages", title: "Packages", icon: Package },
    { tab: "releases", title: "Releases", icon: Boxes },
  ] },
  { label: "Plan", items: [
    { tab: "issues", title: "Issues", icon: ListTodo },
    { tab: "merge-requests", title: "Merge Requests", icon: GitPullRequest },
  ] },
  { label: "Repository", items: [
    { tab: "commits", title: "Commits", icon: GitCommit },
    { tab: "branches", title: "Branches", icon: GitBranch },
    { tab: "lfs", title: "LFS", icon: Database },
  ] },
  { label: "CI/CD", items: [
    { tab: "pipelines", title: "Pipelines", icon: Rocket },
    { tab: "jobs", title: "Jobs", icon: PlayCircle },
    { tab: "runners", title: "Runners", icon: Users },
  ] },
  { label: "Operate", items: [
    { tab: "audit", title: "Audit", icon: ScrollText },
    { tab: "settings", title: "Settings", icon: Settings },
  ] },
];

export function ProjectContextNavigation({
  route,
  currentTab,
  onNavigate,
  t,
}: ProjectContextNavigationProps): JSX.Element {
  const basePath = projectBasePath(route);
  const activeUrl = currentTab === "overview" ? basePath : `${basePath}?tab=${currentTab}`;

  return (
    <>
      <ContextNavigationGroup
        label={t("Projects")}
        items={[{ title: t("Back to projects"), url: "/app/projects", icon: ArrowLeft }]}
        activeUrl="none"
        onNavigate={onNavigate}
      />
      {projectGroups.map((group) => (
        <ContextNavigationGroup
          key={group.label}
          label={t(group.label)}
          items={group.items.map((item) => ({
            title: t(item.title),
            url: item.tab === "overview" ? basePath : `${basePath}?tab=${item.tab}`,
            icon: item.icon,
          }))}
          activeUrl={activeUrl}
          collapsible
          onNavigate={onNavigate}
        />
      ))}
    </>
  );
}
