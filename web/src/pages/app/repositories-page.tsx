import { useEffect, useRef, useState } from "react";
import { useSearchParams } from "react-router";
import { useCreate, useDelete, useList } from "@refinedev/core";
import { Building2, FolderGit2, Plus, ShieldCheck } from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { useI18n } from "@/lib/i18n";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import type { OrganizationView, RepositoryView } from "@/pages/types";
import {
  CreateProjectDialog,
  createProjectDraft,
  type ProjectDraft,
} from "./workspace/create-project-dialog";
import { ProjectWorkspace } from "./workspace/project-workspace";
import { useGitignoreTemplates, useLicenseTemplates } from "./workspace/use-license-templates";

export const AppRepositoriesPage = (): JSX.Element => {
  const { t } = useI18n();
  const [selectedOrg, setSelectedOrg] = useState("");
  const [params, setParams] = useSearchParams();
  const isCreateModalOpen = params.get("create") === "1";
  const setCreateModalOpen = (open: boolean) => {
    const next = new URLSearchParams(params);
    if (open) next.set("create", "1"); else next.delete("create");
    setParams(next);
  };
  const [draft, setDraft] = useState<ProjectDraft>(() => createProjectDraft());
  const [actionError, setActionError] = useState<string | null>(null);
  const submitLock = useRef(false);
  const licenseQuery = useLicenseTemplates();
  const gitignoreQuery = useGitignoreTemplates();
  const { mutate: createRepository, mutation: { isPending: isCreating } } = useCreate<RepositoryView>();
  const { mutate: deleteRepository, mutation: { isPending: isDeleting } } = useDelete<RepositoryView>();
  const orgQuery = useList<OrganizationView>({ resource: "organizations", pagination: { mode: "off" } });
  const orgs = orgQuery.result.data ?? [];

  useEffect(() => {
    if (!selectedOrg && orgs[0]) {
      setSelectedOrg(orgs[0].id);
    }
  }, [orgs, selectedOrg]);

  useEffect(() => {
    if (!draft.ownerOrganizationId && selectedOrg) {
      setDraft((current) => ({ ...current, ownerOrganizationId: selectedOrg }));
    }
  }, [draft.ownerOrganizationId, selectedOrg]);

  const repoQuery = useList<RepositoryView>({
    resource: "my-projects",
    pagination: { mode: "off" },
    meta: { organization_id: selectedOrg },
    queryOptions: { enabled: Boolean(selectedOrg) },
  });

  const repos = repoQuery.result.data ?? [];
  const selectedOrganization = orgs.find((organization) => organization.id === selectedOrg) ?? null;
  const errorMessage = actionError
    ?? (orgQuery.query.error instanceof Error
      ? orgQuery.query.error.message
      : repoQuery.query.error instanceof Error
        ? repoQuery.query.error.message
        : null);
  const isLoading = orgQuery.query.isLoading || repoQuery.query.isLoading;

  const submitCreate = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (submitLock.current || isCreating || isDeleting) return;
    submitLock.current = true;
    setActionError(null);
    if (!draft.ownerOrganizationId) {
      submitLock.current = false;
      setActionError(t("Please select an organization first."));
      return;
    }
    if (draft.licenseTemplate !== "none" && !licenseQuery.templates.some((item) => item.id === draft.licenseTemplate)) {
      submitLock.current = false;
      setActionError(t("Could not load license templates."));
      return;
    }
    if (draft.gitignoreTemplate !== "none" && !gitignoreQuery.templates.some((item) => item.id === draft.gitignoreTemplate)) {
      submitLock.current = false;
      setActionError(t("Could not load .gitignore templates."));
      return;
    }
    createRepository(
      {
        resource: "my-projects",
        values: {
          organization_id: draft.ownerOrganizationId,
          key: draft.key,
          path_key: draft.key,
          name: draft.name,
          description: draft.description || undefined,
          visibility: draft.visibility,
          default_branch: draft.defaultBranch || undefined,
          gitignore_template: draft.gitignoreTemplate === "none" ? undefined : draft.gitignoreTemplate,
          license_template: draft.licenseTemplate === "none" ? undefined : draft.licenseTemplate,
          initialize_with_readme: draft.initializeWithReadme,
          license_copyright_holder: !["mit", "bsd-2-clause", "bsd-3-clause"].includes(draft.licenseTemplate)
            || !draft.licenseCopyrightHolder.trim()
            ? undefined
            : draft.licenseCopyrightHolder.trim(),
        },
      },
      {
        onSuccess: async () => {
          submitLock.current = false;
          const ownerOrganizationId = draft.ownerOrganizationId;
          const ownerChanged = ownerOrganizationId !== selectedOrg;
          setDraft(createProjectDraft(ownerOrganizationId));
          setCreateModalOpen(false);
          if (ownerChanged) {
            setSelectedOrg(ownerOrganizationId);
          } else {
            await repoQuery.query.refetch();
          }
        },
        onError: (error) => {
          submitLock.current = false;
          setActionError(error instanceof Error ? error.message : t("Failed to create project"));
        },
      },
    );
  };

  const submitDelete = (repository: RepositoryView, confirmation: string) => {
    setActionError(null);
    deleteRepository(
      { resource: "my-projects", id: repository.id, meta: { confirmation } },
      {
        onSuccess: async () => { await repoQuery.query.refetch(); },
        onError: (error) => {
          setActionError(error instanceof Error ? error.message : t("Failed to delete project"));
        },
      },
    );
  };

  const copyCloneUrl = async (cloneUrl: string) => {
    try {
      await navigator.clipboard.writeText(cloneUrl);
    } catch {
      setActionError(t("Failed to copy clone URL"));
    }
  };

  const openCreateModal = () => {
    setDraft((current) => ({ ...current, ownerOrganizationId: selectedOrg || orgs[0]?.id || "" }));
    setCreateModalOpen(true);
  };

  return (
    <div className="flex flex-col gap-4 page-enter">
      <section className="overflow-hidden rounded-lg border border-border/80 bg-card shadow-sm" aria-labelledby="projects-title">
        <div className="flex flex-col gap-4 border-b border-border/70 px-4 py-4 sm:px-5 lg:flex-row lg:items-center lg:justify-between">
          <div>
            <p className="mb-1 text-xs font-medium text-muted-foreground">{t("Workspace")} / {t("My Projects")}</p>
            <h1 id="projects-title" className="text-xl font-semibold tracking-tight">{t("Projects")}</h1>
            <p className="mt-1 max-w-2xl text-sm text-muted-foreground">
              {t("Browse organization namespaces, open project repositories, and copy clone URLs from one workspace.")}
            </p>
          </div>
          <Button type="button" size="sm" onClick={openCreateModal}><Plus aria-hidden="true" />{t("New Project")}</Button>
        </div>
        <div className="grid divide-y divide-border/70 sm:grid-cols-3 sm:divide-x sm:divide-y-0">
          <ProjectStatus icon={Building2} label={t("Current scope")} value={selectedOrganization?.name ?? t("No organization selected")} />
          <ProjectStatus icon={ShieldCheck} label={t("Namespace key")} value={selectedOrganization?.key ?? t("N/A")} mono />
          <ProjectStatus icon={FolderGit2} label={t("Available projects")} value={isLoading ? "--" : String(repos.length)} badge />
        </div>
      </section>

      {errorMessage ? <Alert variant="destructive"><AlertDescription>{errorMessage}</AlertDescription></Alert> : null}

      <ProjectWorkspace
        organizations={orgs}
        selectedOrganization={selectedOrganization}
        selectedOrganizationId={selectedOrg}
        repositories={repos}
        isLoading={isLoading}
        hasError={Boolean(errorMessage)}
        isDeleting={isDeleting}
        onSelectOrganization={setSelectedOrg}
        onCreate={openCreateModal}
        onCopy={(cloneUrl) => { void copyCloneUrl(cloneUrl); }}
        onDelete={submitDelete}
      />

      <CreateProjectDialog
        open={isCreateModalOpen}
        organizations={orgs}
        draft={draft}
        isSubmitting={isCreating || isDeleting}
        licenseTemplates={licenseQuery.templates}
        gitignoreTemplates={gitignoreQuery.templates}
        isLoadingLicenses={licenseQuery.query.isLoading}
        isLoadingGitignores={gitignoreQuery.query.isLoading}
        licenseError={licenseQuery.query.error ? t("Could not load license templates.") : null}
        gitignoreError={gitignoreQuery.query.error ? t("Could not load .gitignore templates.") : null}
        onRetryLicenses={() => { void licenseQuery.query.refetch(); }}
        onRetryGitignores={() => { void gitignoreQuery.query.refetch(); }}
        onDraftChange={setDraft}
        onClose={() => setCreateModalOpen(false)}
        onSubmit={submitCreate}
      />
    </div>
  );
};

const ProjectStatus = ({ icon: Icon, label, value, mono = false, badge = false }: { icon: LucideIcon; label: string; value: string; mono?: boolean; badge?: boolean }): JSX.Element => <div className="flex items-center gap-3 px-4 py-3 sm:px-5">
      <Icon className="size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
      <div className="min-w-0">
        <p className="text-xs text-muted-foreground">{label}</p>
        {badge
          ? <Badge variant="secondary" className="mt-0.5">{value}</Badge>
          : <p className={`truncate text-sm font-semibold ${mono ? "font-mono" : ""}`}>{value}</p>}
      </div>
    </div>;
