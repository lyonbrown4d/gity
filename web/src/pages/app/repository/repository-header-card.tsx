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

const TABS: Array<{ value: RepoTab; label: string; icon: LucideIcon }> = [
  { value: "code", label: "Code", icon: Code2 },
  { value: "issues", label: "Issues", icon: ListTodo },
  { value: "merge-requests", label: "Merge Requests", icon: GitPullRequest },
  { value: "wiki", label: "Wiki", icon: BookOpen },
  { value: "pipelines", label: "Pipelines", icon: Rocket },
  { value: "jobs", label: "Jobs", icon: PlayCircle },
  { value: "runners", label: "Runners", icon: Users },
  { value: "commits", label: "Commits", icon: GitCommit },
  { value: "branches", label: "Branches", icon: GitBranch },
  { value: "settings", label: "Settings", icon: Settings },
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
            <ProductEyebrow>{organizationName ?? t("Organization")}</ProductEyebrow>
            <div className="space-y-2">
              <CardTitle className="truncate text-3xl tracking-[-0.04em] md:text-4xl">
                {repository?.name ?? t("Repository")}
              </CardTitle>
              <CardDescription className="max-w-3xl break-all leading-6">
                {repository?.description || t("No description provided.")}
              </CardDescription>
            </div>
            <div className="flex flex-wrap gap-2">
              <ProductStatusBadge>{repository?.visibility ?? t("N/A")}</ProductStatusBadge>
              <ProductStatusBadge icon={GitBranch} variant="secondary">
                {repository?.default_branch ?? "main"}
              </ProductStatusBadge>
              {repository ? <ProductStatusBadge>{repository.key}</ProductStatusBadge> : null}
            </div>
          </div>
          <div className="flex flex-wrap items-center gap-2 lg:justify-end">
            <Button type="button" className="action-pop" onClick={onCopyCloneUrl} disabled={!repository}>
              {t("Copy Clone URL")}
            </Button>
            <Button type="button" variant="outline" className="action-pop" asChild>
              <Link to="/app/repositories">{t("Back to repositories")}</Link>
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent className="relative z-10">
        <Tabs value={activeTab} onValueChange={(value) => onChangeTab(value as RepoTab)}>
          <TabsList className="-mx-1 max-w-full overflow-x-auto">
            {TABS.map((tab) => (
              <TabsTrigger key={tab.value} value={tab.value} className="shrink-0 gap-1.5">
                <tab.icon className="size-3.5" />
                {t(tab.label)}
              </TabsTrigger>
            ))}
          </TabsList>
        </Tabs>
      </CardContent>
    </Card>
  );
};
