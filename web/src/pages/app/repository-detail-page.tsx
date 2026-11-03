import { useMemo } from "react";
import { Link, useNavigate, useParams, useLocation } from "react-router";
import { useCustom, usePermissions } from "@refinedev/core";
import { useI18n } from "@/lib/i18n";
import { useTheme } from "@/lib/theme";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator } from "@/components/ui/breadcrumb";
import { Section } from "@/components/surface/section";
import { RepositoryPinnedSource, parsePinnedSourceTarget } from "@/pages/app/repository/workspace/repository-pinned-source";
import { RepositoryCodeTab } from "@/pages/app/repository/repository-code-tab";
import { RepositoryCreateFileModal } from "@/pages/app/repository/repository-create-file-modal";
import { RepositoryHeaderCard } from "@/pages/app/repository/repository-header-card";
import { RepositoryOverviewTab } from "@/pages/app/repository/repository-overview-tab";
import { buildRepositoryPermissions } from "@/pages/app/repository/repository-permissions";
import type { RepoTab } from "@/pages/app/repository/repository-types";
import { useRepositoryMetaController } from "@/pages/app/repository/use-repository-meta-controller";
import { useRepositorySourceController } from "@/pages/app/repository/use-repository-source-controller";
import type { RawRecord } from "@/pages/app/repository/repository-normalizers";
import { RepositorySecondaryTabContent } from "@/pages/app/repository/workspace/repository-secondary-tab-content";

export const RepositoryDetailPage = (): JSX.Element => {
  const { t } = useI18n();
  const { editorTheme } = useTheme();
  const navigate = useNavigate();
  const location = useLocation();
  const params = useParams<{ organizationId: string; projectId?: string; repoId?: string; jobId?: string }>();
  const organizationId = params.organizationId ?? "";
  const repoId = params.projectId ?? params.repoId ?? "";
  const routeTab = params.jobId ? "jobs" : location.pathname.split("/").filter(Boolean)[4] ?? new URLSearchParams(location.search).get("tab");
  const pinnedSource = parsePinnedSourceTarget(location.search);
  const activeTab: RepoTab = isRepoTab(routeTab) ? routeTab : "overview";
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
    repoId: pinnedSource ? "" : repoId,
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

  const handleChangeTab = (tab: RepoTab) => {
    const base = `/app/projects/${encodeURIComponent(organizationId)}/${encodeURIComponent(repoId)}`;
    navigate(tab === "overview" ? base : `${base}/${tab}`);
  };

  return (
    <div className="flex flex-col gap-3 page-enter">
      <Breadcrumb>
        <BreadcrumbList className="px-1 py-0.5 text-xs">
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
        organizationName={meta.organization?.name}
        repository={meta.repository}
        permissions={repositoryPermissions}
        t={t}
        onCopyCloneUrl={() => void meta.copyCloneUrl()}
      />

      {meta.errorMessage ? (
        <Alert variant="destructive">
          <AlertDescription>{meta.errorMessage}</AlertDescription>
        </Alert>
      ) : null}

      {meta.isLoading ? (
        <Section>
          <p className="py-6 text-center text-sm text-muted-foreground">{t("Loading...")}</p>
        </Section>
      ) : null}

      {!meta.isLoading && !meta.repository ? (
        <Section>
          <p className="py-6 text-center text-sm text-muted-foreground">{t("Project not found in selected organization.")}</p>
        </Section>
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

      {meta.repository && activeTab === "code" && pinnedSource ? (
        <RepositoryPinnedSource key={`${repoId}:${location.search}`} repository={meta.repository} target={pinnedSource}
          t={t} editorTheme={editorTheme} onBrowse={() => handleChangeTab("code")} />
      ) : null}
      {meta.repository && activeTab === "code" && !pinnedSource ? (
        <RepositoryCodeTab
          t={t}
          repository={meta.repository}
          canWrite={repositoryPermissions.repositoryPush}
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

      <RepositorySecondaryTabContent
        activeTab={activeTab}
        organizationId={organizationId}
        repoId={repoId}
        meta={meta}
        permissions={repositoryPermissions}
        t={t}
      />

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

const isRepoTab = (value: string | null | undefined): value is RepoTab =>
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
