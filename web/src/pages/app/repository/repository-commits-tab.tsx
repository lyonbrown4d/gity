import { Badge } from "@/components/ui/badge";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { DataList, DataRow } from "@/components/surface/data-list";
import { Section } from "@/components/surface/section";
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
    <Section
      className="card-enter"
      title={t("Commits")}
      description={t("Recent commit activity in this project.")}
      flush
      actions={
        <Select value={branchFilter} onValueChange={onChangeBranchFilter}>
          <SelectTrigger id="branch-filter" className="h-8 w-44 text-xs" aria-label={t("Branch")}>
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
      }
    >
      {isLoadingCommits ? (
        <div className="px-4 py-6 text-sm text-muted-foreground">{t("Loading commits...")}</div>
      ) : commits.length === 0 ? (
        <div className="px-4 py-6 text-sm text-muted-foreground">{t("No commits found.")}</div>
      ) : (
        <DataList>
          {commits.map((commit) => (
            <DataRow key={commit.commit_sha} className="gap-4">
              <span className="flex min-w-0 flex-1 flex-col gap-1">
                <span className="truncate text-sm font-medium">{commit.message}</span>
                <span className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
                  <Badge variant="outline" className="font-mono text-[11px]">{shortSha(commit.commit_sha)}</Badge>
                  <Badge variant="secondary" className="text-[11px]">{commit.branch_name}</Badge>
                  <span className="truncate">
                    {commit.author_user_id} · {formatTime(commit.created_at)}
                  </span>
                </span>
              </span>
            </DataRow>
          ))}
        </DataList>
      )}
    </Section>
  );
};
