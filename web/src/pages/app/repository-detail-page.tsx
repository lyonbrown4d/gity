import { useEffect, useMemo, useState } from "react";
import { Link, useNavigate, useParams, useSearchParams } from "react-router-dom";
import { useCustom, usePermissions } from "@refinedev/core";
import { useI18n } from "@/lib/i18n";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator } from "@/components/ui/breadcrumb";
import { Card, CardContent } from "@/components/ui/card";
import { RepositoryAuditTab } from "@/pages/app/repository/repository-audit-tab";
import { RepositoryBranchesTab } from "@/pages/app/repository/repository-branches-tab";
import { RepositoryCodeTab } from "@/pages/app/repository/repository-code-tab";
import { RepositoryCommitsTab } from "@/pages/app/repository/repository-commits-tab";
import { RepositoryCreateFileModal } from "@/pages/app/repository/repository-create-file-modal";
import { RepositoryHeaderCard } from "@/pages/app/repository/repository-header-card";
import { RepositoryIssuesTab } from "@/pages/app/repository/repository-issues-tab";
import { RepositoryLFSTab } from "@/pages/app/repository/repository-lfs-tab";
import { RepositoryJobsTab } from "@/pages/app/repository/repository-jobs-tab";
import { RepositoryMergeRequestsTab } from "@/pages/app/repository/repository-merge-requests-tab";
import { RepositoryOverviewTab } from "@/pages/app/repository/repository-overview-tab";
import { RepositoryPackagesTab } from "@/pages/app/repository/repository-packages-tab";
import { buildRepositoryPermissions } from "@/pages/app/repository/repository-permissions";
import { RepositoryPipelinesTab } from "@/pages/app/repository/repository-pipelines-tab";
import { RepositoryReleasesTab } from "@/pages/app/repository/repository-releases-tab";
import { RepositoryRunnersTab } from "@/pages/app/repository/repository-runners-tab";
import { RepositorySettingsTab } from "@/pages/app/repository/repository-settings-tab";
import { RepositoryWikiTab } from "@/pages/app/repository/repository-wiki-tab";
import type { RepoTab } from "@/pages/app/repository/repository-types";
import { useRepositoryMetaController } from "@/pages/app/repository/use-repository-meta-controller";
import { useRepositorySourceController } from "@/pages/app/repository/use-repository-source-controller";
import type { RawRecord } from "@/pages/app/repository/repository-normalizers";

export const RepositoryDetailPage = (): JSX.Element => {
  const { t } = useI18n();
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();
  const params = useParams<{ organizationId: string; projectId?: string; repoId?: string }>();
  const organizationId = params.organizationId ?? "";
  const repoId = params.projectId ?? params.repoId ?? "";
  const initialTab = searchParams.get("tab");
  const [activeTab, setActiveTab] = useState<RepoTab>(isRepoTab(initialTab) ? initialTab : "overview");
  const editorTheme = document.documentElement.classList.contains("dark") ? "vs-dark" : "vs";
  const permissionsQuery = usePermissions<{ isSuperAdmin?: boolean }>({});
  const projectPermissionsQuery = useCustom<RawRecord>({
    url: repoId ? `/projects/${repoId}/permissions` : "/projects/0/permissions",
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
      retry: false,
    },
  });

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
  const repositoryPermissions = useMemo(
    () => buildRepositoryPermissions(meta.organization?.role, Boolean(permissionsQuery.data?.isSuperAdmin), projectPermissionsQuery.result.data),
    [meta.organization?.role, permissionsQuery.data?.isSuperAdmin, projectPermissionsQuery.result.data],
  );

  useEffect(() => {
    const tab = searchParams.get("tab");
    setActiveTab(isRepoTab(tab) ? tab : "overview");
  }, [searchParams]);

  const handleChangeTab = (tab: RepoTab) => {
    setActiveTab(tab);
    const next = new URLSearchParams(searchParams);
    if (tab === "overview") {
      next.delete("tab");
    } else {
      next.set("tab", tab);
    }
    setSearchParams(next, { replace: true });
  };

  return (
    <div className="flex flex-col gap-4 page-enter">
      <Breadcrumb>
        <BreadcrumbList className="rounded-xl border border-border/70 bg-card/70 px-3 py-2 shadow-sm backdrop-blur">
          <BreadcrumbItem>
            <BreadcrumbLink asChild>
              <Link to="/app/projects">{t("Projects")}</Link>
            </BreadcrumbLink>
          </BreadcrumbItem>
          <BreadcrumbSeparator />
          <BreadcrumbItem>
            <span className="max-w-[40vw] truncate text-muted-foreground">
              {meta.organization?.name ?? (organizationId || t("Organization"))}
            </span>
          </BreadcrumbItem>
          <BreadcrumbSeparator />
          <BreadcrumbItem>
            <BreadcrumbPage className="max-w-[50vw] truncate font-medium">
              {meta.repository?.name ?? (repoId || t("Project"))}
            </BreadcrumbPage>
          </BreadcrumbItem>
        </BreadcrumbList>
      </Breadcrumb>

      <RepositoryHeaderCard
        activeTab={activeTab}
        organizationName={meta.organization?.name}
        repository={meta.repository}
        permissions={repositoryPermissions}
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

      {meta.repository && activeTab === "overview" ? (
        <RepositoryOverviewTab
          repoId={repoId}
          repository={meta.repository}
          branches={meta.branches}
          commits={meta.commits}
          isLoadingCommits={meta.isLoadingCommits}
          permissions={repositoryPermissions}
          t={t}
          onOpenTab={handleChangeTab}
          onCopyCloneUrl={() => void meta.copyCloneUrl()}
        />
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
          permissions={repositoryPermissions}
          t={t}
          onError={meta.setActionError}
        />
      ) : null}

      {meta.repository && activeTab === "merge-requests" ? (
        <RepositoryMergeRequestsTab
          organizationId={organizationId}
          repoId={repoId}
          branches={meta.branches}
          defaultBranch={meta.repository.default_branch}
          permissions={repositoryPermissions}
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
          defaultBranch={meta.repository.default_branch}
          permissions={repositoryPermissions}
          t={t}
          onError={meta.setActionError}
        />
      ) : null}

      {meta.repository && activeTab === "jobs" ? (
        <RepositoryJobsTab
          repoId={repoId}
          permissions={repositoryPermissions}
          t={t}
          onError={meta.setActionError}
        />
      ) : null}

      {meta.repository && activeTab === "wiki" ? (
        <RepositoryWikiTab
          repoId={repoId}
          permissions={repositoryPermissions}
          t={t}
          onError={meta.setActionError}
        />
      ) : null}

      {meta.repository && activeTab === "packages" ? (
        <RepositoryPackagesTab
          repoId={repoId}
          permissions={repositoryPermissions}
          t={t}
          onError={meta.setActionError}
        />
      ) : null}

      {meta.repository && activeTab === "releases" ? (
        <RepositoryReleasesTab
          repoId={repoId}
          defaultBranch={meta.repository.default_branch}
          permissions={repositoryPermissions}
          t={t}
          onError={meta.setActionError}
        />
      ) : null}

      {meta.repository && activeTab === "lfs" ? (
        <RepositoryLFSTab
          repoId={repoId}
          repository={meta.repository}
          permissions={repositoryPermissions}
          t={t}
          onError={meta.setActionError}
        />
      ) : null}

      {meta.repository && activeTab === "runners" ? (
        <RepositoryRunnersTab
          repoId={repoId}
          permissions={repositoryPermissions}
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
          defaultBranch={meta.repository.default_branch}
          newBranchName={meta.newBranchName}
          isLoadingBranches={meta.isLoadingBranches}
          isCreatingBranch={meta.isCreatingBranch}
          isUpdatingBranch={meta.isUpdatingBranch}
          isDeletingBranch={meta.isDeletingBranch}
          permissions={repositoryPermissions}
          onChangeNewBranchName={meta.setNewBranchName}
          onSubmitCreateBranch={(event) => void meta.submitCreateBranch(event)}
          onToggleBranchProtection={(branch, protect) => void meta.toggleBranchProtection(branch, protect)}
          onUpdateBranchProtection={(branch, patch) => void meta.updateBranchProtection(branch, patch)}
          onDeleteBranch={(branch) => void meta.removeBranch(branch)}
        />
      ) : null}

      {meta.repository && activeTab === "settings" ? (
        <RepositorySettingsTab
          repository={meta.repository}
          permissions={repositoryPermissions}
          t={t}
          isDeleting={meta.isDeleting}
          onError={meta.setActionError}
          onDelete={meta.submitDelete}
        />
      ) : null}

      {meta.repository && activeTab === "audit" ? (
        <RepositoryAuditTab
          repoId={repoId}
          t={t}
          onError={meta.setActionError}
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
  value === "overview"
  || value === "code"
  || value === "issues"
  || value === "merge-requests"
  || value === "wiki"
  || value === "packages"
  || value === "releases"
  || value === "lfs"
  || value === "pipelines"
  || value === "jobs"
  || value === "runners"
  || value === "commits"
  || value === "branches"
  || value === "audit"
  || value === "settings";
