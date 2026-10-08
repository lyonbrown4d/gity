import { RepositoryAuditTab } from "@/pages/app/repository/repository-audit-tab";
import { RepositoryBranchesTab } from "@/pages/app/repository/repository-branches-tab";
import { RepositoryCommitsTab } from "@/pages/app/repository/repository-commits-tab";
import { RepositoryIssuesTab } from "@/pages/app/repository/repository-issues-tab";
import { RepositoryJobsTab } from "@/pages/app/repository/repository-jobs-tab";
import { RepositoryLFSTab } from "@/pages/app/repository/repository-lfs-tab";
import { RepositoryMergeRequestsTab } from "@/pages/app/repository/repository-merge-requests-tab";
import { RepositoryPackagesTab } from "@/pages/app/repository/repository-packages-tab";
import { RepositoryPipelinesTab } from "@/pages/app/repository/repository-pipelines-tab";
import { RepositoryReleasesTab } from "@/pages/app/repository/repository-releases-tab";
import { RepositoryRunnersTab } from "@/pages/app/repository/repository-runners-tab";
import { RepositorySettingsTab } from "@/pages/app/repository/repository-settings-tab";
import type { RepositoryPermissions } from "@/pages/app/repository/repository-permissions";
import type { RepoTab } from "@/pages/app/repository/repository-types";
import type { useRepositoryMetaController } from "@/pages/app/repository/use-repository-meta-controller";
import { RepositoryWikiTab } from "@/pages/app/repository/repository-wiki-tab";

interface RepositorySecondaryTabContentProps {
  activeTab: RepoTab;
  organizationId: string;
  repoId: string;
  meta: ReturnType<typeof useRepositoryMetaController>;
  permissions: RepositoryPermissions;
  t: (text: string) => string;
}

export const RepositorySecondaryTabContent = ({
  activeTab,
  organizationId,
  repoId,
  meta,
  permissions,
  t,
}: RepositorySecondaryTabContentProps): JSX.Element | null => {
  if (!meta.repository) {
    return null;
  }

  if (activeTab === "issues") {
    return <RepositoryIssuesTab organizationId={organizationId} repoId={repoId} permissions={permissions} t={t} onError={meta.setActionError} />;
  }
  if (activeTab === "merge-requests") {
    return (
      <RepositoryMergeRequestsTab
        organizationId={organizationId}
        repoId={repoId}
        branches={meta.branches}
        defaultBranch={meta.repository.default_branch}
        permissions={permissions}
        t={t}
        onError={meta.setActionError}
        onMerged={async () => {
          await Promise.all([meta.loadBranches(), meta.loadCommits()]);
        }}
      />
    );
  }
  if (activeTab === "pipelines") {
    return <RepositoryPipelinesTab repoId={repoId} defaultBranch={meta.repository.default_branch} permissions={permissions} t={t} onError={meta.setActionError} />;
  }
  if (activeTab === "jobs") {
    return <RepositoryJobsTab repoId={repoId} permissions={permissions} t={t} onError={meta.setActionError} />;
  }
  if (activeTab === "wiki") {
    return <RepositoryWikiTab repoId={repoId} permissions={permissions} t={t} onError={meta.setActionError} />;
  }
  if (activeTab === "packages") {
    return <RepositoryPackagesTab repoId={repoId} permissions={permissions} t={t} onError={meta.setActionError} />;
  }
  if (activeTab === "releases") {
    return <RepositoryReleasesTab repoId={repoId} defaultBranch={meta.repository.default_branch} permissions={permissions} t={t} onError={meta.setActionError} />;
  }
  if (activeTab === "lfs") {
    return <RepositoryLFSTab repoId={repoId} repository={meta.repository} permissions={permissions} t={t} onError={meta.setActionError} />;
  }
  if (activeTab === "runners") {
    return <RepositoryRunnersTab repoId={repoId} permissions={permissions} t={t} onError={meta.setActionError} />;
  }
  if (activeTab === "commits") {
    return (
      <RepositoryCommitsTab
        t={t}
        branches={meta.branches}
        commits={meta.commits}
        branchFilter={meta.branchFilter}
        isLoadingCommits={meta.isLoadingCommits}
        onChangeBranchFilter={meta.setBranchFilter}
      />
    );
  }
  if (activeTab === "branches") {
    return (
      <RepositoryBranchesTab
        t={t}
        branches={meta.branches}
        defaultBranch={meta.repository.default_branch}
        newBranchName={meta.newBranchName}
        isLoadingBranches={meta.isLoadingBranches}
        isCreatingBranch={meta.isCreatingBranch}
        isUpdatingBranch={meta.isUpdatingBranch}
        isDeletingBranch={meta.isDeletingBranch}
        permissions={permissions}
        onChangeNewBranchName={meta.setNewBranchName}
        onSubmitCreateBranch={(event) => void meta.submitCreateBranch(event)}
        onToggleBranchProtection={(branch, protect) => void meta.toggleBranchProtection(branch, protect)}
        onUpdateBranchProtection={(branch, update) => void meta.updateBranchProtection(branch, update)}
        onDeleteBranch={(branch) => void meta.removeBranch(branch)}
      />
    );
  }
  if (activeTab === "settings") {
    return <RepositorySettingsTab repository={meta.repository} permissions={permissions} t={t} isDeleting={meta.isDeleting} onError={meta.setActionError} onDelete={meta.submitDelete} />;
  }
  if (activeTab === "audit") {
    return <RepositoryAuditTab repoId={repoId} t={t} onError={meta.setActionError} />;
  }
  return null;
};
