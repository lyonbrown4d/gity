import { Link } from "react-router-dom";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
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

const TABS: RepoTab[] = ["code", "issues", "merge-requests", "wiki", "jobs", "runners", "commits", "branches", "settings"];

export const RepositoryHeaderCard = ({
  activeTab,
  organizationName,
  repository,
  t,
  onChangeTab,
  onCopyCloneUrl,
}: RepositoryHeaderCardProps): JSX.Element => {
  return (
    <Card className="card-enter">
      <CardHeader>
        <div className="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
          <div className="min-w-0 space-y-2">
            <CardTitle className="truncate text-xl">
              {organizationName ?? t("Organization")} / {repository?.name ?? t("Repository")}
            </CardTitle>
            <CardDescription className="break-all">
              {repository?.description || t("No description provided.")}
            </CardDescription>
            <div className="flex flex-wrap gap-2">
              <Badge variant="outline">{repository?.visibility ?? t("N/A")}</Badge>
              <Badge variant="secondary">
                {t("default branch:")} {repository?.default_branch ?? "main"}
              </Badge>
              {repository ? <Badge variant="outline">{repository.key}</Badge> : null}
            </div>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Button type="button" variant="outline" className="action-pop" onClick={onCopyCloneUrl} disabled={!repository}>
              {t("Copy Clone URL")}
            </Button>
            <Button type="button" variant="outline" className="action-pop" asChild>
              <Link to="/app/repositories">{t("Back to repositories")}</Link>
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <div className="flex flex-wrap gap-2">
          {TABS.map((tab) => (
            <Button
              key={tab}
              type="button"
              variant={activeTab === tab ? "default" : "outline"}
              size="sm"
              onClick={() => onChangeTab(tab)}
              className="action-pop"
            >
              {t(tabLabel(tab))}
            </Button>
          ))}
        </div>
      </CardContent>
    </Card>
  );
};

const tabLabel = (tab: RepoTab): string => {
  if (tab === "merge-requests") {
    return "Merge Requests";
  }
  return tab[0].toUpperCase() + tab.slice(1);
};
