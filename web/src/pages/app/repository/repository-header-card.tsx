import { Link } from "react-router-dom";
import {
  BookOpen,
  Code2,
  GitBranch,
  GitCommit,
  GitPullRequest,
  ListTodo,
  PlayCircle,
  Rocket,
  Settings,
  Users,
  type LucideIcon,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { ProductEyebrow, ProductStatusBadge } from "@/components/ui/product";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import type { RepositoryView } from "@/pages/types";
import type { RepoTab } from "./repository-types";

interface RepositoryHeaderCardProps {
  activeTab: RepoTab;
  organizationName?: string;
  repository: RepositoryView | null;
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
      { value: "settings", label: "Settings", icon: Settings },
    ],
  },
];

export const RepositoryHeaderCard = ({
  activeTab,
  organizationName,
  repository,
  t,
  onChangeTab,
  onCopyCloneUrl,
}: RepositoryHeaderCardProps): JSX.Element => {
  return (
    <Card className="gity-hero card-enter">
      <CardHeader className="relative z-10 pb-4">
        <div className="flex flex-col gap-5 lg:flex-row lg:items-start lg:justify-between">
          <div className="min-w-0 space-y-4">
            <ProductEyebrow>{t("Project workspace")}</ProductEyebrow>
            <div className="space-y-2">
              <CardTitle className="truncate text-3xl tracking-[-0.04em] md:text-4xl">
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
            </div>
          </div>
          <div className="flex flex-wrap items-center gap-2 lg:justify-end">
            <Button type="button" className="action-pop" onClick={onCopyCloneUrl} disabled={!repository}>
              {t("Copy Clone URL")}
            </Button>
            <Button type="button" variant="outline" className="action-pop" asChild>
              <Link to="/app/projects">{t("Back to projects")}</Link>
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent className="relative z-10">
        <Tabs value={activeTab} onValueChange={(value) => onChangeTab(value as RepoTab)} className="space-y-3">
          <div className="grid gap-3 xl:grid-cols-[1fr_1fr_1fr_1.25fr_0.8fr]">
            {TAB_GROUPS.map((group) => (
              <div key={group.label} className="rounded-2xl border border-border/70 bg-background/45 p-2">
                <p className="px-2 pb-2 text-[11px] font-semibold uppercase tracking-[0.18em] text-muted-foreground">
                  {t(group.label)}
                </p>
                <TabsList className="h-auto w-full flex-wrap justify-start gap-1 bg-transparent p-0">
                  {group.items.map((tab) => (
                    <TabsTrigger key={tab.value} value={tab.value} className="h-8 shrink-0 gap-1.5 rounded-xl px-2.5">
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
