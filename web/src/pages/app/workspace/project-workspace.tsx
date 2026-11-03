import { Link } from "react-router";
import { ArrowRight, Building2, Copy, FolderGit2, GitBranch, Plus, Trash2 } from "lucide-react";
import { ConfirmAction } from "@/components/common/confirm-action";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import { DataList, DataRow } from "@/components/surface/data-list";
import { Section } from "@/components/surface/section";
import { useI18n } from "@/lib/i18n";
import type { OrganizationView, RepositoryView } from "@/pages/types";

interface ProjectWorkspaceProps {
  organizations: OrganizationView[];
  selectedOrganization: OrganizationView | null;
  selectedOrganizationId: string;
  repositories: RepositoryView[];
  isLoading: boolean;
  hasError: boolean;
  isDeleting: boolean;
  onSelectOrganization: (organizationId: string) => void;
  onCreate: () => void;
  onCopy: (cloneUrl: string) => void;
  onDelete: (repository: RepositoryView, confirmation: string) => void;
}

export const ProjectWorkspace = ({
  organizations,
  selectedOrganization,
  selectedOrganizationId,
  repositories,
  isLoading,
  hasError,
  isDeleting,
  onSelectOrganization,
  onCreate,
  onCopy,
  onDelete,
}: ProjectWorkspaceProps): JSX.Element => {
  const { t } = useI18n();

  return (
    <div className="grid items-start gap-4 xl:grid-cols-[260px_minmax(0,1fr)]">
      <Section
        title={(
          <span className="flex items-center gap-2">
            <Building2 className="size-4 text-muted-foreground" aria-hidden="true" />
            {t("Organization")}
          </span>
        )}
        description={t("Choose the namespace that owns the projects.")}
        actions={(
          <Button type="button" variant="outline" size="sm" onClick={onCreate}>
            <Plus aria-hidden="true" />{t("New Project")}
          </Button>
        )}
      >
        <div className="space-y-3">
          <div className="space-y-1.5">
            <Label htmlFor="organization-select" className="text-xs">{t("Organization")}</Label>
            <Select value={selectedOrganizationId} onValueChange={onSelectOrganization}>
              <SelectTrigger id="organization-select">
                <SelectValue placeholder={t("Organization")} />
              </SelectTrigger>
              <SelectContent>
                {organizations.map((organization) => (
                  <SelectItem key={organization.id} value={organization.id}>{organization.name}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <dl className="divide-y divide-border border-y border-border">
            <ScopeDetail label={t("Namespace key")} value={selectedOrganization?.key ?? "--"} mono />
            <ScopeDetail label={t("Available projects")} value={isLoading ? "--" : String(repositories.length)} />
          </dl>
        </div>
      </Section>

      <Section
        title={t("Project directory")}
        description={selectedOrganization
          ? `${selectedOrganization.name} / ${selectedOrganization.key}`
          : t("Select an organization to view projects.")}
        actions={(
          <Button type="button" size="sm" onClick={onCreate}>
            <Plus aria-hidden="true" />{t("New Project")}
          </Button>
        )}
        flush
      >
        {isLoading ? <DirectorySkeleton /> : null}
        {!isLoading && repositories.length > 0 ? (
          <DataList>
            {repositories.map((repository) => (
              <ProjectRow
                key={repository.id}
                repository={repository}
                isDeleting={isDeleting}
                onCopy={onCopy}
                onDelete={onDelete}
              />
            ))}
          </DataList>
        ) : null}
        {!isLoading && repositories.length === 0 && !hasError ? <EmptyDirectory onCreate={onCreate} /> : null}
      </Section>
    </div>
  );
};

const ProjectRow = ({
  repository,
  isDeleting,
  onCopy,
  onDelete,
}: {
  repository: RepositoryView;
  isDeleting: boolean;
  onCopy: (cloneUrl: string) => void;
  onDelete: (repository: RepositoryView, confirmation: string) => void;
}): JSX.Element => {
  const { t } = useI18n();
  const projectUrl = `/app/projects/${repository.organization_id}/${repository.id}`;

  return (
    <DataRow className="items-start gap-3 sm:items-center">
      <FolderGit2 className="mt-0.5 size-4 shrink-0 text-muted-foreground sm:mt-0" aria-hidden="true" />
      <div className="min-w-0 flex-1">
        <div className="flex flex-wrap items-center gap-1.5">
          <Link to={projectUrl} className="truncate text-[13px] font-medium underline-offset-4 hover:underline">{repository.name}</Link>
          <Badge variant="secondary">{repository.visibility}</Badge>
          {repository.status !== "active" ? <Badge variant="outline">{repository.status}</Badge> : null}
        </div>
        <p className="truncate font-mono text-[11px] text-muted-foreground">{repository.full_path}</p>
        <p className="line-clamp-1 text-xs text-muted-foreground">
          {repository.description || t("No description provided.")}
        </p>
      </div>

      <div className="hidden min-w-0 shrink-0 text-right lg:block">
        <div className="flex items-center justify-end gap-1.5 text-[11px] text-muted-foreground">
          <GitBranch className="size-3" aria-hidden="true" />
          <span>{repository.default_branch}</span>
          <span aria-hidden="true">/</span>
          <span className="font-mono">{repository.key}</span>
        </div>
        <p className="mt-0.5 max-w-[220px] truncate font-mono text-[11px]" title={repository.clone_http_url}>{repository.clone_http_url}</p>
      </div>

      <div className="flex shrink-0 items-center gap-1">
        <Button type="button" variant="ghost" size="sm" onClick={() => onCopy(repository.clone_http_url)}>
          <Copy aria-hidden="true" />{t("Copy")}
        </Button>
        <Button variant="ghost" size="sm" asChild>
          <Link to={projectUrl}>{t("Open")}<ArrowRight aria-hidden="true" /></Link>
        </Button>
        <ConfirmAction
          title={t("Delete project \"{name}\"?").replace("{name}", repository.name)}
          description={t("This action cannot be undone.")}
          confirmLabel={t("Delete")}
          cancelLabel={t("Cancel")}
          verificationLabel={t("Type {path} to confirm deletion.").replace("{path}", repository.full_path)}
          verificationValue={repository.full_path}
          verificationPlaceholder={repository.full_path}
          onConfirm={(verification) => onDelete(repository, verification ?? "")}
        >
          <Button type="button" variant="ghost" size="icon" disabled={isDeleting} aria-label={t("Delete")}>
            <Trash2 className="text-destructive" aria-hidden="true" />
          </Button>
        </ConfirmAction>
      </div>
    </DataRow>
  );
};

const ScopeDetail = ({ label, value, mono = false }: { label: string; value: string; mono?: boolean }): JSX.Element => <div className="flex items-center justify-between gap-3 py-2">
      <dt className="text-xs text-muted-foreground">{label}</dt>
      <dd className={`truncate text-[13px] ${mono ? "font-mono" : "font-semibold"}`}>{value}</dd>
    </div>;

const DirectorySkeleton = (): JSX.Element => <div className="divide-y divide-border">
      {[0, 1, 2].map((item) => (
        <div key={item} className="px-4 py-3"><Skeleton className="h-10 w-full" /></div>
      ))}
    </div>;

const EmptyDirectory = ({ onCreate }: { onCreate: () => void }): JSX.Element => {
  const { t } = useI18n();
  return (
    <div className="flex min-h-48 flex-col items-center justify-center gap-2 px-6 py-8 text-center">
      <FolderGit2 className="size-5 text-muted-foreground" aria-hidden="true" />
      <p className="text-sm font-medium">{t("No projects available.")}</p>
      <p className="max-w-md text-xs text-muted-foreground">{t("Create a project in this organization to start pushing code.")}</p>
      <Button type="button" size="sm" onClick={onCreate}><Plus aria-hidden="true" />{t("New Project")}</Button>
    </div>
  );
};
