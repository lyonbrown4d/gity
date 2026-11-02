import { Link } from "react-router";
import { ArrowRight, Building2, Copy, FolderGit2, GitBranch, Plus, Trash2 } from "lucide-react";
import { ConfirmAction } from "@/components/common/confirm-action";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
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

export function ProjectWorkspace({
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
}: ProjectWorkspaceProps): JSX.Element {
  const { t } = useI18n();

  return (
    <div className="grid gap-4 xl:grid-cols-[260px_minmax(0,1fr)]">
      <Card className="h-fit overflow-hidden shadow-sm">
        <CardHeader className="border-b border-border/70 p-4">
          <div className="flex items-center gap-2">
            <Building2 className="size-4 text-muted-foreground" aria-hidden="true" />
            <CardTitle className="text-base">{t("Organization")}</CardTitle>
          </div>
          <CardDescription>{t("Choose the namespace that owns the projects.")}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4 p-4">
          <div className="space-y-2">
            <Label htmlFor="organization-select">{t("Organization")}</Label>
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
          <dl className="divide-y divide-border/70 border-y border-border/70 text-sm">
            <ScopeDetail label={t("Namespace key")} value={selectedOrganization?.key ?? "--"} mono />
            <ScopeDetail label={t("Available projects")} value={isLoading ? "--" : String(repositories.length)} />
          </dl>
          <Button type="button" onClick={onCreate} className="w-full">
            <Plus aria-hidden="true" />{t("New Project")}
          </Button>
        </CardContent>
      </Card>

      <Card className="overflow-hidden shadow-sm">
        <CardHeader className="flex-row items-center justify-between gap-3 space-y-0 border-b border-border/70 p-4 sm:px-5">
          <div className="min-w-0">
            <CardTitle className="text-base">{t("Project directory")}</CardTitle>
            <CardDescription className="mt-1 truncate">
              {selectedOrganization
                ? `${selectedOrganization.name} / ${selectedOrganization.key}`
                : t("Select an organization to view projects.")}
            </CardDescription>
          </div>
          <Button type="button" variant="outline" size="sm" onClick={onCreate}>
            <Plus aria-hidden="true" />{t("New Project")}
          </Button>
        </CardHeader>
        <CardContent className="p-0">
          {isLoading ? <DirectorySkeleton /> : null}
          {!isLoading && repositories.length > 0 ? repositories.map((repository) => (
            <ProjectRow
              key={repository.id}
              repository={repository}
              isDeleting={isDeleting}
              onCopy={onCopy}
              onDelete={onDelete}
            />
          )) : null}
          {!isLoading && repositories.length === 0 && !hasError ? <EmptyDirectory onCreate={onCreate} /> : null}
        </CardContent>
      </Card>
    </div>
  );
}

function ProjectRow({
  repository,
  isDeleting,
  onCopy,
  onDelete,
}: {
  repository: RepositoryView;
  isDeleting: boolean;
  onCopy: (cloneUrl: string) => void;
  onDelete: (repository: RepositoryView, confirmation: string) => void;
}): JSX.Element {
  const { t } = useI18n();
  const projectUrl = `/app/projects/${repository.organization_id}/${repository.id}`;

  return (
    <article
      aria-label={repository.name}
      className="grid gap-3 border-b border-border/70 px-4 py-4 last:border-b-0 hover:bg-muted/30 sm:px-5 lg:grid-cols-[minmax(0,1fr)_minmax(260px,0.75fr)_auto] lg:items-center"
    >
      <div className="flex min-w-0 gap-3">
        <FolderGit2 className="mt-0.5 size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-2">
            <Link to={projectUrl} className="truncate text-sm font-semibold underline-offset-4 hover:underline">{repository.name}</Link>
            <Badge variant="secondary">{repository.visibility}</Badge>
            {repository.status !== "active" ? <Badge variant="outline">{repository.status}</Badge> : null}
          </div>
          <p className="mt-1 truncate text-xs text-muted-foreground">{repository.full_path}</p>
          <p className="mt-1 line-clamp-1 text-sm text-muted-foreground">
            {repository.description || t("No description provided.")}
          </p>
        </div>
      </div>

      <div className="min-w-0 border-l-0 border-border/70 lg:border-l lg:pl-4">
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <GitBranch className="size-3" aria-hidden="true" />
          <span>{repository.default_branch}</span>
          <span aria-hidden="true">/</span>
          <span className="font-mono">{repository.key}</span>
        </div>
        <p className="mt-1 truncate font-mono text-xs" title={repository.clone_http_url}>{repository.clone_http_url}</p>
      </div>

      <div className="flex items-center gap-1 lg:justify-end">
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
    </article>
  );
}

function ScopeDetail({ label, value, mono = false }: { label: string; value: string; mono?: boolean }): JSX.Element {
  return <div className="flex items-center justify-between gap-3 py-2.5"><dt className="text-xs text-muted-foreground">{label}</dt><dd className={mono ? "truncate font-mono font-medium" : "font-semibold"}>{value}</dd></div>;
}

function DirectorySkeleton(): JSX.Element {
  return <div className="space-y-px p-3">{[0, 1, 2].map((item) => <Skeleton key={item} className="h-20 w-full" />)}</div>;
}

function EmptyDirectory({ onCreate }: { onCreate: () => void }): JSX.Element {
  const { t } = useI18n();
  return <div className="flex min-h-48 flex-col items-center justify-center gap-3 px-6 py-8 text-center"><FolderGit2 className="size-6 text-muted-foreground" aria-hidden="true" /><div><p className="text-sm font-medium">{t("No projects available.")}</p><p className="mt-1 text-sm text-muted-foreground">{t("Create a project in this organization to start pushing code.")}</p></div><Button type="button" size="sm" onClick={onCreate}><Plus aria-hidden="true" />{t("New Project")}</Button></div>;
}
