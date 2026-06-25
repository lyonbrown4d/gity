import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { useCreate, useDelete, useList } from "@refinedev/core";
import { ArrowRight, Building2, Copy, FolderGit2, GitBranch, Plus, ShieldCheck, Trash2 } from "lucide-react";
import { useI18n } from "@/lib/i18n";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ConfirmAction } from "@/components/common/confirm-action";
import { FormDialog as Modal } from "@/components/common/form-dialog";
import { ProductHero, ProductStatusBadge } from "@/components/ui/product";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import { Skeleton } from "@/components/ui/skeleton";
import type { OrganizationView, RepositoryView } from "@/pages/types";

export function AppRepositoriesPage(): JSX.Element {
  const { t } = useI18n();
  const [selectedOrg, setSelectedOrg] = useState<string>("");
  const [isCreateModalOpen, setCreateModalOpen] = useState(false);
  const [createOwnerOrg, setCreateOwnerOrg] = useState("");
  const [key, setKey] = useState("");
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [defaultBranch, setDefaultBranch] = useState("main");
  const [visibility, setVisibility] = useState("private");
  const [gitignoreTemplate, setGitignoreTemplate] = useState("none");
  const [licenseTemplate, setLicenseTemplate] = useState("none");
  const [actionError, setActionError] = useState<string | null>(null);

  const { mutate: createRepository, mutation: { isPending: isCreating } } = useCreate<RepositoryView>();
  const { mutate: deleteRepository, mutation: { isPending: isDeleting } } = useDelete<RepositoryView>();

  const orgQuery = useList<OrganizationView>({
    resource: "organizations",
  });
  const orgs = orgQuery.result.data ?? [];

  useEffect(() => {
    if (!selectedOrg && orgs[0]) {
      setSelectedOrg(orgs[0].id);
    }
  }, [orgs, selectedOrg]);

  useEffect(() => {
    if (!createOwnerOrg && selectedOrg) {
      setCreateOwnerOrg(selectedOrg);
    }
  }, [createOwnerOrg, selectedOrg]);

  const repoQuery = useList<RepositoryView>({
    resource: "my-projects",
    meta: {
      organization_id: selectedOrg,
    },
    queryOptions: {
      enabled: Boolean(selectedOrg),
    },
  });

  const repos = repoQuery.result.data ?? [];
  const selectedOrganization = orgs.find((org) => org.id === selectedOrg) ?? null;
  const errorMessage = actionError
    ?? (orgQuery.query.error instanceof Error
      ? orgQuery.query.error.message
      : repoQuery.query.error instanceof Error
        ? repoQuery.query.error.message
        : null);
  const isLoading = orgQuery.query.isLoading || repoQuery.query.isLoading;

  const submitCreate = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setActionError(null);
    if (!createOwnerOrg) {
      setActionError(t("Please select an organization first."));
      return;
    }
    createRepository(
      {
        resource: "my-projects",
        values: {
          organization_id: createOwnerOrg,
          key,
          path_key: key,
          name,
          description: description || undefined,
          visibility,
          default_branch: defaultBranch || undefined,
          gitignore_template: gitignoreTemplate === "none" ? undefined : gitignoreTemplate,
          license_template: licenseTemplate === "none" ? undefined : licenseTemplate,
        },
      },
      {
        onSuccess: async () => {
          setKey("");
          setName("");
          setDescription("");
          setDefaultBranch("main");
          setVisibility("private");
          setGitignoreTemplate("none");
          setLicenseTemplate("none");
          const ownerChanged = createOwnerOrg !== selectedOrg;
          if (ownerChanged) {
            setSelectedOrg(createOwnerOrg);
          }
          setCreateModalOpen(false);
          if (!ownerChanged) {
            await repoQuery.query.refetch();
          }
        },
        onError: (error) => {
          setActionError(error instanceof Error ? error.message : t("Failed to create project"));
        },
      },
    );
  };

  const submitDelete = (repo: RepositoryView, confirmation: string) => {
    setActionError(null);
    deleteRepository(
      {
        resource: "my-projects",
        id: repo.id,
        meta: { confirmation },
      },
      {
        onSuccess: async () => {
          await repoQuery.query.refetch();
        },
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
    setCreateOwnerOrg(selectedOrg || orgs[0]?.id || "");
    setCreateModalOpen(true);
  };

  return (
    <div className="flex flex-col gap-5 page-enter">
      <ProductHero
        className="card-enter"
        eyebrow={t("Workspace")}
        title={t("Projects")}
        description={t("Browse organization namespaces, open project repositories, and copy clone URLs from one workspace.")}
        aside={(
          <div className="rounded-xl border border-border/80 bg-background/75 p-4 shadow-sm backdrop-blur">
            <p className="text-xs font-semibold uppercase tracking-[0.2em] text-muted-foreground">
              {t("Current scope")}
            </p>
            <div className="flex flex-col gap-2 pt-4">
              <p className="truncate text-lg font-semibold">
                {selectedOrganization?.name ?? t("No organization selected")}
              </p>
              <div className="flex flex-wrap items-center gap-2">
                <Badge variant="secondary">{selectedOrganization?.key ?? t("N/A")}</Badge>
                <Badge variant="outline">
                  {repos.length} {t("projects")}
                </Badge>
              </div>
            </div>
          </div>
        )}
        contentClassName="flex flex-col gap-4"
      >
        <div className="flex flex-wrap items-center gap-2">
          <ProductStatusBadge icon={Building2} variant="secondary">
            {selectedOrganization?.name ?? t("Organization")}
          </ProductStatusBadge>
          <ProductStatusBadge icon={FolderGit2} variant="secondary">
            {repos.length} {t("projects")}
          </ProductStatusBadge>
          <ProductStatusBadge icon={ShieldCheck}>
            {selectedOrg ? t("organization selected") : t("no organization")}
          </ProductStatusBadge>
        </div>

        {errorMessage ? (
          <Alert variant="destructive">
            <AlertDescription>{errorMessage}</AlertDescription>
          </Alert>
        ) : null}
      </ProductHero>

      <div className="grid gap-4 xl:grid-cols-[280px_minmax(0,1fr)]">
        <Card className="card-enter">
          <CardHeader>
            <div className="flex items-center gap-3">
              <div className="flex size-10 items-center justify-center rounded-2xl bg-primary/10 text-primary">
                <Building2 className="size-5" />
              </div>
              <div className="min-w-0">
                <CardTitle>{t("Organization")}</CardTitle>
                <CardDescription>{t("Choose the namespace that owns the projects.")}</CardDescription>
              </div>
            </div>
          </CardHeader>
          <CardContent className="flex flex-col gap-4">
            <div className="flex flex-col gap-2">
              <Label htmlFor="organization-select">{t("Organization")}</Label>
              <Select value={selectedOrg} onValueChange={setSelectedOrg}>
                <SelectTrigger id="organization-select">
                  <SelectValue placeholder={t("Organization")} />
                </SelectTrigger>
                <SelectContent>
                  {orgs.map((org) => (
                    <SelectItem key={org.id} value={org.id}>
                      {org.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <Separator />

            <div className="flex flex-col gap-3">
              <div className="rounded-lg border border-border/80 bg-muted/35 p-3">
                <p className="text-xs text-muted-foreground">{t("Namespace key")}</p>
                <p className="truncate font-mono text-sm font-semibold">
                  {selectedOrganization?.key ?? "--"}
                </p>
              </div>
              <div className="rounded-lg border border-border/80 bg-muted/35 p-3">
                <p className="text-xs text-muted-foreground">{t("Available projects")}</p>
                <p className="text-2xl font-semibold">{isLoading ? "--" : repos.length}</p>
              </div>
            </div>

            <Button type="button" onClick={openCreateModal} className="w-full action-pop">
              <Plus data-icon="inline-start" />
              {t("New Project")}
            </Button>
          </CardContent>
        </Card>

        <Card className="overflow-hidden card-enter">
          <CardHeader className="gap-4 border-b border-border/70">
            <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
              <div className="flex min-w-0 items-center gap-3">
                <div className="flex size-10 items-center justify-center rounded-2xl bg-primary/10 text-primary">
                  <FolderGit2 className="size-5" />
                </div>
                <div className="min-w-0">
                  <CardTitle>{t("Project directory")}</CardTitle>
                  <CardDescription>
                    {selectedOrganization
                      ? `${selectedOrganization.name} / ${selectedOrganization.key}`
                      : t("Select an organization to view projects.")}
                  </CardDescription>
                </div>
              </div>
              <Button type="button" variant="outline" onClick={openCreateModal} className="action-pop">
                <Plus data-icon="inline-start" />
                {t("New Project")}
              </Button>
            </div>
          </CardHeader>
          <CardContent className="p-0">
            {isLoading ? (
              <div className="flex flex-col gap-3 p-4">
                <Skeleton className="h-24 w-full" />
                <Skeleton className="h-24 w-full" />
                <Skeleton className="h-24 w-full" />
              </div>
            ) : null}

            {!isLoading ? (
              <div className="flex flex-col">
                {repos.map((repo) => (
                  <div
                    key={repo.id}
                    className="group grid gap-4 border-b border-border/70 p-4 transition-colors last:border-b-0 hover:bg-muted/40 md:grid-cols-[minmax(0,1fr)_minmax(260px,0.58fr)]"
                  >
                    <div className="flex min-w-0 gap-3">
                      <div className="flex size-11 shrink-0 items-center justify-center rounded-xl border border-border/80 bg-background text-primary shadow-sm">
                        <FolderGit2 className="size-5" />
                      </div>
                      <div className="flex min-w-0 flex-col gap-2">
                        <div className="flex min-w-0 flex-wrap items-center gap-2">
                          <Link
                            to={`/app/projects/${repo.organization_id}/${repo.id}`}
                            className="min-w-0 truncate text-base font-semibold underline-offset-4 hover:underline"
                          >
                            {repo.name}
                          </Link>
                          <Badge variant="secondary" className="shrink-0">
                            {repo.visibility}
                          </Badge>
                          {repo.status !== "active" ? (
                            <Badge variant="outline" className="shrink-0">
                              {repo.status}
                            </Badge>
                          ) : null}
                        </div>
                        <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
                          <span className="truncate font-medium text-foreground">
                            {repo.full_path}
                          </span>
                          <span className="rounded-md bg-muted px-1.5 py-0.5 font-mono text-[11px]">
                            {repo.key}
                          </span>
                          <span className="inline-flex items-center gap-1">
                            <GitBranch className="size-3" />
                            {repo.default_branch}
                          </span>
                        </div>
                        <p className="break-words text-sm leading-6 text-muted-foreground">
                          {repo.description || t("No description provided.")}
                        </p>
                      </div>
                    </div>

                    <div className="flex flex-col gap-3 rounded-xl border border-border/80 bg-background/70 p-3">
                      <div className="flex items-center justify-between gap-3">
                        <p className="text-xs font-medium text-muted-foreground">{t("Clone URL")}</p>
                        <Button
                          type="button"
                          variant="ghost"
                          size="sm"
                          className="h-7 px-2 action-pop"
                          onClick={() => copyCloneUrl(repo.clone_http_url)}
                        >
                          <Copy data-icon="inline-start" />
                          {t("Copy")}
                        </Button>
                      </div>
                      <p className="break-all rounded-lg bg-muted/55 px-3 py-2 font-mono text-xs">
                        {repo.clone_http_url}
                      </p>
                      <div className="flex flex-wrap items-center gap-2 md:justify-end">
                        <Button type="button" variant="outline" size="sm" className="action-pop" asChild>
                          <Link to={`/app/projects/${repo.organization_id}/${repo.id}`}>
                            {t("Open")}
                            <ArrowRight data-icon="inline-end" />
                          </Link>
                        </Button>
                        <ConfirmAction
                          title={t("Delete project \"{name}\"?").replace("{name}", repo.name)}
                          description={t("This action cannot be undone.")}
                          confirmLabel={t("Delete")}
                          cancelLabel={t("Cancel")}
                          verificationLabel={t("Type {path} to confirm deletion.").replace("{path}", repo.full_path)}
                          verificationValue={repo.full_path}
                          verificationPlaceholder={repo.full_path}
                          onConfirm={(verification) => submitDelete(repo, verification ?? "")}
                        >
                          <Button
                            type="button"
                            variant="destructive"
                            size="sm"
                            className="action-pop"
                            disabled={isDeleting}
                          >
                            <Trash2 data-icon="inline-start" />
                            {t("Delete")}
                          </Button>
                        </ConfirmAction>
                      </div>
                    </div>
                  </div>
                ))}
                {repos.length === 0 && !errorMessage ? (
                  <div className="flex flex-col items-start gap-3 p-6">
                    <div className="flex size-12 items-center justify-center rounded-2xl bg-muted text-muted-foreground">
                      <FolderGit2 className="size-6" />
                    </div>
                    <div className="flex flex-col gap-1">
                      <p className="font-medium">{t("No projects available.")}</p>
                      <p className="text-sm text-muted-foreground">
                        {t("Create a project in this organization to start pushing code.")}
                      </p>
                    </div>
                    <Button type="button" onClick={openCreateModal} className="action-pop">
                      <Plus data-icon="inline-start" />
                      {t("New Project")}
                    </Button>
                  </div>
                ) : null}
              </div>
            ) : null}
          </CardContent>
        </Card>
      </div>

      <Modal open={isCreateModalOpen} onClose={() => setCreateModalOpen(false)} title={t("Create Project")}>
        <form className="grid gap-3 md:grid-cols-2" onSubmit={submitCreate}>
          <div className="flex flex-col gap-2 md:col-span-2">
            <Label htmlFor="repo-owner">{t("Owner")}</Label>
            <Select value={createOwnerOrg} onValueChange={setCreateOwnerOrg} required>
              <SelectTrigger id="repo-owner">
                <SelectValue placeholder={t("Owner")} />
              </SelectTrigger>
              <SelectContent>
                {orgs.map((org) => (
                  <SelectItem key={org.id} value={org.id}>
                    {org.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="flex flex-col gap-2">
            <Label htmlFor="repo-key">{t("Project key")}</Label>
            <Input
              id="repo-key"
              placeholder={t("Project key")}
              value={key}
              onChange={(event) => setKey(event.target.value)}
              required
            />
          </div>
          <div className="flex flex-col gap-2">
            <Label htmlFor="repo-name">{t("Project name")}</Label>
            <Input
              id="repo-name"
              placeholder={t("Project Name")}
              value={name}
              onChange={(event) => setName(event.target.value)}
              required
            />
          </div>
          <div className="flex flex-col gap-2 md:col-span-2">
            <Label htmlFor="repo-description">{t("Description")}</Label>
            <Input
              id="repo-description"
              placeholder={t("Description (optional)")}
              value={description}
              onChange={(event) => setDescription(event.target.value)}
            />
          </div>
          <div className="flex flex-col gap-2">
            <Label htmlFor="repo-default-branch">{t("Default branch")}</Label>
            <Input
              id="repo-default-branch"
              placeholder={t("Default branch")}
              value={defaultBranch}
              onChange={(event) => setDefaultBranch(event.target.value)}
            />
          </div>
          <div className="flex flex-col gap-2">
            <Label htmlFor="repo-visibility">{t("Visibility")}</Label>
            <Select value={visibility} onValueChange={setVisibility}>
              <SelectTrigger id="repo-visibility">
                <SelectValue placeholder={t("Visibility")} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="private">{t("private")}</SelectItem>
                <SelectItem value="internal">{t("internal")}</SelectItem>
                <SelectItem value="public">{t("public")}</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="flex flex-col gap-2">
            <Label htmlFor="repo-gitignore">{t("Add .gitignore")}</Label>
            <Select value={gitignoreTemplate} onValueChange={setGitignoreTemplate}>
              <SelectTrigger id="repo-gitignore">
                <SelectValue placeholder={t("Add .gitignore")} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="none">{t("None")}</SelectItem>
                <SelectItem value="rust">{t("Rust")}</SelectItem>
                <SelectItem value="node">{t("Node")}</SelectItem>
                <SelectItem value="python">{t("Python")}</SelectItem>
                <SelectItem value="go">{t("Go")}</SelectItem>
                <SelectItem value="java">{t("Java")}</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="flex flex-col gap-2">
            <Label htmlFor="repo-license">{t("Add license")}</Label>
            <Select value={licenseTemplate} onValueChange={setLicenseTemplate}>
              <SelectTrigger id="repo-license">
                <SelectValue placeholder={t("Add license")} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="none">{t("None")}</SelectItem>
                <SelectItem value="mit">{t("MIT License")}</SelectItem>
                <SelectItem value="apache-2.0">{t("Apache License 2.0")}</SelectItem>
                <SelectItem value="gpl-3.0">{t("GNU GPLv3")}</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="flex items-center gap-2 md:col-span-2 md:justify-end">
            <Button type="button" variant="outline" onClick={() => setCreateModalOpen(false)}>
              {t("Cancel")}
            </Button>
            <Button type="submit" disabled={isCreating || isDeleting}>
              {isCreating ? t("Creating...") : t("Create")}
            </Button>
          </div>
        </form>
      </Modal>
    </div>
  );
}


