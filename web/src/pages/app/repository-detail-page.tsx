import { useEffect, useState } from "react";
import { Link, useNavigate, useParams, useSearchParams } from "react-router-dom";
import { useI18n } from "@/lib/i18n";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Card, CardContent } from "@/components/ui/card";
import { RepositoryBranchesTab } from "@/pages/app/repository/repository-branches-tab";
import { RepositoryCodeTab } from "@/pages/app/repository/repository-code-tab";
import { RepositoryCommitsTab } from "@/pages/app/repository/repository-commits-tab";
import { RepositoryCreateFileModal } from "@/pages/app/repository/repository-create-file-modal";
import { RepositoryHeaderCard } from "@/pages/app/repository/repository-header-card";
import { RepositoryIssuesTab } from "@/pages/app/repository/repository-issues-tab";
import { RepositoryLFSTab } from "@/pages/app/repository/repository-lfs-tab";
import { RepositoryJobsTab } from "@/pages/app/repository/repository-jobs-tab";
import { RepositoryMergeRequestsTab } from "@/pages/app/repository/repository-merge-requests-tab";
import { RepositoryPackagesTab } from "@/pages/app/repository/repository-packages-tab";
import { RepositoryPipelinesTab } from "@/pages/app/repository/repository-pipelines-tab";
import { RepositoryRunnersTab } from "@/pages/app/repository/repository-runners-tab";
import { RepositorySettingsTab } from "@/pages/app/repository/repository-settings-tab";
import { RepositoryWikiTab } from "@/pages/app/repository/repository-wiki-tab";
import type { RepoTab } from "@/pages/app/repository/repository-types";
import { useRepositoryMetaController } from "@/pages/app/repository/use-repository-meta-controller";
import { useRepositorySourceController } from "@/pages/app/repository/use-repository-source-controller";

export const RepositoryDetailPage = (): JSX.Element => {
  const { t } = useI18n();
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();
  const params = useParams<{ organizationId: string; projectId?: string; repoId?: string }>();
  const organizationId = params.organizationId ?? "";
  const repoId = params.projectId ?? params.repoId ?? "";
  const initialTab = searchParams.get("tab");
  const [activeTab, setActiveTab] = useState<RepoTab>(isRepoTab(initialTab) ? initialTab : "code");
  const editorTheme = document.documentElement.classList.contains("dark") ? "vs-dark" : "vs";

  const meta = useRepositoryMetaController({
    organizationId,
    repoId,
    t,
    onDeleted: () => navigate("/app/projects", { replace: true }),
  });
  const source = useRepositorySourceController({
    repoId,
    activeTab,
    defaultBranch: meta.repository?.default_branch ?? null,
    branches: meta.branches,
    t,
    onError: meta.setActionError,
    refreshBranches: meta.loadBranches,
    refreshCommits: meta.loadCommits,
  });

  useEffect(() => {
    const tab = searchParams.get("tab");
    if (!tab) {
      return;
    }
    if (isRepoTab(tab)) {
      setActiveTab(tab);
    }
  }, [searchParams]);

  const handleChangeTab = (tab: RepoTab) => {
    setActiveTab(tab);
    const next = new URLSearchParams(searchParams);
    if (tab === "code") {
      next.delete("tab");
    } else {
      next.set("tab", tab);
    }
    setSearchParams(next, { replace: true });
  };

  return (
    <div className="space-y-4 page-enter">
      <div className="text-sm text-muted-foreground">
        <Link to="/app/projects" className="underline underline-offset-4">
          {t("My Projects")}
        </Link>
      </div>

      <RepositoryHeaderCard
        activeTab={activeTab}
        organizationName={meta.organization?.name}
        repository={meta.repository}
        t={t}
        onChangeTab={handleChangeTab}
        onCopyCloneUrl={() => void meta.copyCloneUrl()}
      />

      {meta.errorMessage ? (
        <Alert variant="destructive">
          <AlertDescription>{meta.errorMessage}</AlertDescription>
        </Alert>
      ) : null}

      {meta.isLoading ? <p className="text-sm text-muted-foreground">{t("Loading...")}</p> : null}

      {!meta.isLoading && !meta.repository ? (
        <Card>
          <CardContent className="pt-6">
            <p className="text-sm text-muted-foreground">{t("Project not found in selected organization.")}</p>
          </CardContent>
        </Card>
      ) : null}

      {meta.repository && activeTab === "code" ? (
        <RepositoryCodeTab
          t={t}
          repository={meta.repository}
          branches={meta.branches}
          codeBranch={source.codeBranch}
          treeNodes={source.treeNodes}
          selectedBlob={source.selectedBlob}
          readmePreview={source.readmePreview}
          readmePath={source.readmePath}
          languages={source.languages}
          isLoadingTree={source.isLoadingTree}
          isLoadingLanguages={source.isLoadingLanguages}
          searchQuery={source.searchQuery}
          searchPath={source.searchPath}
          isLoadingSearch={source.isLoadingSearch}
          searchMatchCase={source.searchMatchCase}
          searchLine={source.searchLine}
          searchColumn={source.searchColumn}
          searchMatchLength={source.searchMatchLength}
          searchJumpRequest={source.searchJumpRequest}
          searchTargetPath={source.searchTargetPath}
          searchRegex={source.searchRegex}
          searchResults={source.searchResults}
          editorTheme={editorTheme}
          onChangeCodeBranch={source.changeCodeBranch}
          onOpenCreateFile={source.openCreateFileModal}
          onOpenFile={(path) => void source.openFile(path)}
          onToggleTreeDirectory={(path) => void source.toggleTreeDirectory(path)}
          onOpenFileAtLine={(path, lineNumber, column, matchLength) => source.openFileAtLine(path, lineNumber, column, matchLength)}
          onSearch={(query) => void source.searchCode(query)}
          setSearchQuery={source.setSearchQuery}
          setSearchPath={source.setSearchPath}
          setSearchMatchCase={source.setSearchMatchCase}
          setSearchRegex={source.setSearchRegex}
          onRefreshLanguages={() => void source.loadLanguages(source.codeBranch)}
        />
      ) : null}

      {meta.repository && activeTab === "issues" ? (
        <RepositoryIssuesTab
          organizationId={organizationId}
          repoId={repoId}
          t={t}
          onError={meta.setActionError}
        />
      ) : null}

      {meta.repository && activeTab === "merge-requests" ? (
        <RepositoryMergeRequestsTab
          repoId={repoId}
          branches={meta.branches}
          defaultBranch={meta.repository.default_branch}
          t={t}
          onError={meta.setActionError}
          onMerged={async () => {
            await Promise.all([meta.loadBranches(), meta.loadCommits()]);
          }}
        />
      ) : null}

      {meta.repository && activeTab === "pipelines" ? (
        <RepositoryPipelinesTab
          repoId={repoId}
          t={t}
          onError={meta.setActionError}
        />
      ) : null}

      {meta.repository && activeTab === "jobs" ? (
        <RepositoryJobsTab
          repoId={repoId}
          t={t}
          onError={meta.setActionError}
        />
      ) : null}

      {meta.repository && activeTab === "wiki" ? (
        <RepositoryWikiTab
          repoId={repoId}
          t={t}
          onError={meta.setActionError}
        />
      ) : null}

      {meta.repository && activeTab === "packages" ? (
        <RepositoryPackagesTab
          repoId={repoId}
          t={t}
          onError={meta.setActionError}
        />
      ) : null}

      {meta.repository && activeTab === "lfs" ? (
        <RepositoryLFSTab
          repoId={repoId}
          repository={meta.repository}
          t={t}
          onError={meta.setActionError}
        />
      ) : null}

      {meta.repository && activeTab === "runners" ? (
        <RepositoryRunnersTab
          repoId={repoId}
          t={t}
          onError={meta.setActionError}
        />
      ) : null}

      {meta.repository && activeTab === "commits" ? (
        <RepositoryCommitsTab
          t={t}
          branches={meta.branches}
          commits={meta.commits}
          branchFilter={meta.branchFilter}
          isLoadingCommits={meta.isLoadingCommits}
          onChangeBranchFilter={meta.setBranchFilter}
        />
      ) : null}

      {meta.repository && activeTab === "branches" ? (
        <RepositoryBranchesTab
          t={t}
          branches={meta.branches}
          newBranchName={meta.newBranchName}
          isLoadingBranches={meta.isLoadingBranches}
          isCreatingBranch={meta.isCreatingBranch}
          isUpdatingBranch={meta.isUpdatingBranch}
          onChangeNewBranchName={meta.setNewBranchName}
          onSubmitCreateBranch={(event) => void meta.submitCreateBranch(event)}
          onToggleBranchProtection={(branch, protect) => void meta.toggleBranchProtection(branch, protect)}
        />
      ) : null}

      {meta.repository && activeTab === "settings" ? (
        <RepositorySettingsTab
          repository={meta.repository}
          t={t}
          isDeleting={meta.isDeleting}
          onDelete={meta.submitDelete}
        />
      ) : null}

      <RepositoryCreateFileModal
        open={source.isCreateFileModalOpen}
        t={t}
        editorTheme={editorTheme}
        branches={meta.branches}
        newFileBranch={source.newFileBranch}
        newFilePath={source.newFilePath}
        newFileMessage={source.newFileMessage}
        newFileContent={source.newFileContent}
        isCreatingFile={source.isCreatingFile}
        onClose={() => source.setCreateFileModalOpen(false)}
        onChangeNewFileBranch={source.setNewFileBranch}
        onChangeNewFilePath={source.setNewFilePath}
        onChangeNewFileMessage={source.setNewFileMessage}
        onChangeNewFileContent={source.setNewFileContent}
        onSubmit={(event) => void source.submitCreateFileCommit(event)}
      />
    </div>
  );
};

const isRepoTab = (value: string | null): value is RepoTab =>
  value === "code"
  || value === "issues"
  || value === "merge-requests"
  || value === "wiki"
  || value === "packages"
  || value === "lfs"
  || value === "pipelines"
  || value === "jobs"
  || value === "runners"
  || value === "commits"
  || value === "branches"
  || value === "settings";
