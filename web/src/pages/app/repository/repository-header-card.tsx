import { Link } from "react-router-dom";
import {
  Activity,
  ArrowLeft,
  BookOpen,
  Code2,
  Copy,
  Database,
  GitBranch,
  GitCommit,
  GitPullRequest,
  ListTodo,
  Package,
  PlayCircle,
  Rocket,
  ScrollText,
  Settings,
  ShieldCheck,
  Users,
  type LucideIcon,
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { ProductStatusBadge } from "@/components/ui/product";
import { Separator } from "@/components/ui/separator";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import type { RepositoryView } from "@/pages/types";
import type { RepositoryPermissions } from "./repository-permissions";
import type { RepoTab } from "./repository-types";

interface RepositoryHeaderCardProps {
  activeTab: RepoTab;
  organizationName?: string;
  repository: RepositoryView | null;
  permissions: RepositoryPermissions;
  t: (text: string) => string;
  onChangeTab: (tab: RepoTab) => void;
  onCopyCloneUrl: () => void;
}

const TAB_GROUPS: Array<{
  label: string;
  items: Array<{ value: RepoTab; label: string; icon: LucideIcon }>;
}> = [
  {
    label: "Project",
    items: [
      { value: "overview", label: "Overview", icon: Activity },
      { value: "code", label: "Code", icon: Code2 },
      { value: "wiki", label: "Wiki", icon: BookOpen },
      { value: "packages", label: "Packages", icon: Package },
      { value: "releases", label: "Releases", icon: Rocket },
    ],
  },
  {
    label: "Plan",
    items: [
      { value: "issues", label: "Issues", icon: ListTodo },
      { value: "merge-requests", label: "Merge Requests", icon: GitPullRequest },
    ],
  },
  {
    label: "Repository",
    items: [
      { value: "commits", label: "Commits", icon: GitCommit },
      { value: "branches", label: "Branches", icon: GitBranch },
      { value: "lfs", label: "LFS", icon: Database },
    ],
  },
  {
    label: "CI/CD",
    items: [
      { value: "pipelines", label: "Pipelines", icon: Rocket },
      { value: "jobs", label: "Jobs", icon: PlayCircle },
      { value: "runners", label: "Runners", icon: Users },
    ],
  },
  {
    label: "Operate",
    items: [
      { value: "audit", label: "Audit", icon: ScrollText },
      { value: "settings", label: "Settings", icon: Settings },
    ],
  },
];

export const RepositoryHeaderCard = ({
  activeTab,
  organizationName,
  repository,
  permissions,
  t,
  onChangeTab,
  onCopyCloneUrl,
}: RepositoryHeaderCardProps): JSX.Element => {
  const projectMark = (repository?.key || repository?.name || "PR").slice(0, 2).toUpperCase();
  const namespacePath = repository?.full_path
    ?? `${organizationName ?? t("Organization")} / ${repository?.key ?? t("Project")}`;

  return (
    <Card className="gity-hero overflow-hidden card-enter">
      <CardHeader className="relative z-10 gap-5 pb-5">
        <div className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_minmax(300px,0.42fr)] xl:items-start">
          <div className="flex min-w-0 gap-4">
            <div className="flex size-14 shrink-0 items-center justify-center rounded-2xl border border-border/80 bg-background/80 text-lg font-semibold text-primary shadow-sm">
              {projectMark}
            </div>
            <div className="flex min-w-0 flex-col gap-3">
              <div className="flex flex-wrap items-center gap-2 text-sm text-muted-foreground">
                <span className="truncate font-medium text-foreground">
                  {organizationName ?? t("Organization")}
                </span>
                <span>/</span>
                <span className="truncate font-mono text-xs">
                  {repository?.key ?? t("Project")}
                </span>
              </div>
              <div className="flex flex-col gap-3">
                <CardTitle className="truncate text-3xl md:text-4xl">
                  {repository?.name ?? t("Project")}
                </CardTitle>
                <CardDescription className="max-w-3xl break-words leading-6">
                  {repository?.description || t("No description provided.")}
                </CardDescription>
              </div>
              <div className="flex flex-wrap gap-2">
                <ProductStatusBadge variant="secondary">
                  {namespacePath}
                </ProductStatusBadge>
                <ProductStatusBadge>{repository?.visibility ?? t("N/A")}</ProductStatusBadge>
                <ProductStatusBadge icon={GitBranch} variant="secondary">
                  {repository?.default_branch ?? "main"}
                </ProductStatusBadge>
                <ProductStatusBadge icon={ShieldCheck} variant={permissions.canWrite ? "default" : "secondary"}>
                  {t("Role")}: {t(permissions.roleLabel)}
                </ProductStatusBadge>
                {repository?.status && repository.status !== "active" ? (
                  <Badge variant="outline">{repository.status}</Badge>
                ) : null}
              </div>
            </div>
          </div>

          <div className="flex flex-col gap-3 rounded-xl border border-border/80 bg-background/75 p-3 shadow-sm backdrop-blur">
            <div className="flex items-center justify-between gap-3">
              <div className="min-w-0">
                <p className="text-xs font-medium text-muted-foreground">{t("Clone URL")}</p>
                <p className="truncate font-mono text-xs text-foreground">
                  {repository?.clone_http_url ?? "--"}
                </p>
              </div>
              <Button type="button" size="sm" className="shrink-0 action-pop" onClick={onCopyCloneUrl} disabled={!repository}>
                <Copy data-icon="inline-start" />
                {t("Copy")}
              </Button>
            </div>
            <Separator />
            <div className="flex flex-wrap items-center gap-2">
              <Button type="button" variant="outline" size="sm" className="action-pop" asChild>
                <Link to="/app/projects">
                  <ArrowLeft data-icon="inline-start" />
                  {t("Back to projects")}
                </Link>
              </Button>
              <Badge variant="secondary" className="font-mono">
                {repository?.uuid?.slice(0, 8) ?? "--------"}
              </Badge>
            </div>
          </div>
        </div>
      </CardHeader>
      <CardContent className="relative z-10 flex flex-col gap-4">
        <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
          <div className="rounded-xl border border-border/80 bg-background/70 p-3">
            <p className="text-xs text-muted-foreground">{t("Default branch")}</p>
            <p className="truncate font-semibold">{repository?.default_branch ?? "main"}</p>
          </div>
          <div className="rounded-xl border border-border/80 bg-background/70 p-3">
            <p className="text-xs text-muted-foreground">{t("Visibility")}</p>
            <p className="truncate font-semibold">{repository?.visibility ?? t("N/A")}</p>
          </div>
          <div className="rounded-xl border border-border/80 bg-background/70 p-3">
            <p className="text-xs text-muted-foreground">{t("Access role")}</p>
            <p className="truncate font-semibold">{t(permissions.roleLabel)}</p>
          </div>
          <div className="rounded-xl border border-border/80 bg-background/70 p-3">
            <p className="text-xs text-muted-foreground">{t("Status")}</p>
            <p className="truncate font-semibold">{repository?.status ?? t("Loading...")}</p>
          </div>
        </div>

        <Tabs value={activeTab} onValueChange={(value) => onChangeTab(value as RepoTab)} className="flex flex-col gap-3">
          <div className="grid gap-3 xl:grid-cols-[1.15fr_1fr_1fr_1.25fr_0.8fr]">
            {TAB_GROUPS.map((group) => (
              <div key={group.label} className="rounded-xl border border-border/80 bg-background/65 p-2 shadow-sm">
                <p className="px-2 pb-2 text-[11px] font-semibold uppercase tracking-[0.12em] text-muted-foreground">
                  {t(group.label)}
                </p>
                <TabsList className="h-auto w-full flex-wrap justify-start gap-1 bg-transparent p-0 shadow-none backdrop-blur-0">
                  {group.items.map((tab) => (
                    <TabsTrigger key={tab.value} value={tab.value} className="h-8 shrink-0 gap-1.5 rounded-md px-2.5">
                      <tab.icon className="size-3.5" />
                      {t(tab.label)}
                    </TabsTrigger>
                  ))}
                </TabsList>
              </div>
            ))}
          </div>
        </Tabs>
      </CardContent>
    </Card>
  );
};
