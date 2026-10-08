import type { RepositoryBranchView, RepositoryCommitView, RepositoryView } from "@/pages/types";
import type { RepositoryPermissions } from "./repository-permissions";
import type { RepoTab } from "./repository-types";
import { RepositoryOverviewWorkspace } from "./workspace/repository-overview-workspace";

interface RepositoryOverviewTabProps {
  repoId: string;
  repository: RepositoryView;
  branches: RepositoryBranchView[];
  commits: RepositoryCommitView[];
  isLoadingCommits: boolean;
  permissions: RepositoryPermissions;
  t: (text: string) => string;
  onOpenTab: (tab: RepoTab) => void;
  onCopyCloneUrl: () => void;
}

export const RepositoryOverviewTab = (props: RepositoryOverviewTabProps): JSX.Element => (
  <RepositoryOverviewWorkspace {...props} />
);
