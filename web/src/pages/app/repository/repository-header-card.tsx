import { Link } from "react-router-dom";
import {
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
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { ProductStatusBadge } from "@/components/ui/product";
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
  return (
    <Card className="gity-hero card-enter">
      <CardHeader className="relative z-10 pb-4">
        <div className="flex flex-col gap-5 lg:flex-row lg:items-start lg:justify-between">
          <div className="flex min-w-0 flex-col gap-4">
            <div className="flex flex-col gap-2">
              <CardTitle className="truncate text-3xl md:text-4xl">
                {repository?.name ?? t("Project")}
              </CardTitle>
              <CardDescription className="max-w-3xl break-all leading-6">
                {repository?.description || t("No description provided.")}
              </CardDescription>
            </div>
            <div className="flex flex-wrap gap-2">
              <ProductStatusBadge variant="secondary">
                {organizationName ?? t("Organization")} / {repository?.key ?? t("Project")}
              </ProductStatusBadge>
              <ProductStatusBadge>{repository?.visibility ?? t("N/A")}</ProductStatusBadge>
              <ProductStatusBadge icon={GitBranch} variant="secondary">
                {repository?.default_branch ?? "main"}
              </ProductStatusBadge>
              <ProductStatusBadge icon={ShieldCheck} variant={permissions.canWrite ? "default" : "secondary"}>
                {t("Role")}: {t(permissions.roleLabel)}
              </ProductStatusBadge>
            </div>
          </div>
          <div className="flex flex-wrap items-center gap-2 lg:justify-end">
            <Button type="button" className="action-pop" onClick={onCopyCloneUrl} disabled={!repository}>
              <Copy className="size-4" />
              {t("Copy Clone URL")}
            </Button>
            <Button type="button" variant="outline" className="action-pop" asChild>
              <Link to="/app/projects">
                <ArrowLeft className="size-4" />
                {t("Back to projects")}
              </Link>
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent className="relative z-10">
        <Tabs value={activeTab} onValueChange={(value) => onChangeTab(value as RepoTab)} className="space-y-3">
          <div className="grid gap-3 xl:grid-cols-[1.15fr_1fr_1fr_1.25fr_0.8fr]">
            {TAB_GROUPS.map((group) => (
              <div key={group.label} className="rounded-lg border border-border/80 bg-background/65 p-2">
                <p className="px-2 pb-2 text-[11px] font-semibold uppercase text-muted-foreground">
                  {t(group.label)}
                </p>
                <TabsList className="h-auto w-full flex-wrap justify-start gap-1 bg-transparent p-0">
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
