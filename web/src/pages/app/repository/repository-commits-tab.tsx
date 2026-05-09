import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import type { RepositoryBranchView, RepositoryCommitView } from "@/pages/types";
import { formatTime, shortSha } from "./repository-utils";

interface RepositoryCommitsTabProps {
  t: (text: string) => string;
  branches: RepositoryBranchView[];
  commits: RepositoryCommitView[];
  branchFilter: string;
  isLoadingCommits: boolean;
  onChangeBranchFilter: (branch: string) => void;
}

export const RepositoryCommitsTab = ({
  t,
  branches,
  commits,
  branchFilter,
  isLoadingCommits,
  onChangeBranchFilter,
}: RepositoryCommitsTabProps): JSX.Element => {
  return (
    <Card className="card-enter">
      <CardHeader>
        <div className="flex flex-col gap-3 md:flex-row md:items-end md:justify-between">
          <div>
            <CardTitle>{t("Commits")}</CardTitle>
            <CardDescription>{t("Recent commit activity in this project.")}</CardDescription>
          </div>
          <div className="space-y-2">
            <Label htmlFor="branch-filter">{t("Branch")}</Label>
            <Select value={branchFilter} onValueChange={onChangeBranchFilter}>
              <SelectTrigger id="branch-filter">
                <SelectValue placeholder={t("Branch")} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">{t("All branches")}</SelectItem>
                {branches.map((branch) => (
                  <SelectItem key={branch.name} value={branch.name}>
                    {branch.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        {isLoadingCommits ? <p className="text-sm text-muted-foreground">{t("Loading commits...")}</p> : null}
        {commits.map((commit) => (
          <div key={commit.commit_sha} className="rounded-md border p-3">
            <div className="flex flex-wrap items-center gap-2">
              <Badge variant="outline">{shortSha(commit.commit_sha)}</Badge>
              <Badge variant="secondary">{commit.branch_name}</Badge>
            </div>
            <p className="mt-2 text-sm font-medium">{commit.message}</p>
            <p className="mt-1 text-xs text-muted-foreground">
              {commit.author_user_id} · {formatTime(commit.created_at)}
            </p>
          </div>
        ))}
        {!isLoadingCommits && commits.length === 0 ? (
          <p className="text-sm text-muted-foreground">{t("No commits found.")}</p>
        ) : null}
      </CardContent>
    </Card>
  );
};
