import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { useCreate, useDelete, useList } from "@refinedev/core";
import { Copy, FolderGit2, GitBranch, Plus, ShieldCheck, Trash2 } from "lucide-react";
import { useI18n } from "@/lib/i18n";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Modal } from "@/components/ui/modal";
import { ProductHero, ProductStatusBadge } from "@/components/ui/product";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
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

  const { mutate: createRepository, isLoading: isCreating } = useCreate<RepositoryView>();
  const { mutate: deleteRepository, isLoading: isDeleting } = useDelete<RepositoryView>();

  const orgQuery = useList<OrganizationView>({
    resource: "organizations",
  });
  const orgs = orgQuery.data?.data ?? [];

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
    resource: "my-repositories",
    meta: {
      organization_id: selectedOrg,
    },
    queryOptions: {
      enabled: Boolean(selectedOrg),
    },
  });

  const repos = repoQuery.data?.data ?? [];
  const errorMessage = actionError
    ?? (orgQuery.error instanceof Error
      ? orgQuery.error.message
      : repoQuery.error instanceof Error
        ? repoQuery.error.message
        : null);
  const isLoading = orgQuery.isLoading || repoQuery.isLoading;

  const submitCreate = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setActionError(null);
    if (!createOwnerOrg) {
      setActionError(t("Please select an organization first."));
      return;
    }
    createRepository(
      {
        resource: "my-repositories",
        values: {
          organization_id: createOwnerOrg,
          key,
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
            await repoQuery.refetch();
          }
        },
        onError: (error) => {
          setActionError(error instanceof Error ? error.message : t("Failed to create repository"));
        },
      },
    );
  };

  const submitDelete = (repo: RepositoryView) => {
    const confirmText = t("Delete repository \"{name}\"?").replace("{name}", repo.name);
    if (!window.confirm(confirmText)) {
      return;
    }
    setActionError(null);
    deleteRepository(
      {
        resource: "my-repositories",
        id: repo.id,
      },
      {
        onSuccess: async () => {
          await repoQuery.refetch();
        },
        onError: (error) => {
          setActionError(error instanceof Error ? error.message : t("Failed to delete repository"));
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
    <div className="space-y-4 page-enter">
      <ProductHero
        className="card-enter"
        eyebrow={t("Workspace")}
        title={t("My Repositories")}
        description={t("Create, clone, and manage repositories in your organizations.")}
        aside={(
            <Button type="button" onClick={openCreateModal} className="h-10 action-pop">
              <Plus className="size-4" />
              {t("New Repository")}
            </Button>
        )}
        contentClassName="space-y-4"
      >
          <div className="grid gap-3 md:grid-cols-[220px_1fr] md:items-end">
            <div className="space-y-2">
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
            <div className="flex flex-wrap items-center gap-2">
              <ProductStatusBadge icon={FolderGit2} variant="secondary">
                {repos.length} {t("repos")}
              </ProductStatusBadge>
              <ProductStatusBadge icon={ShieldCheck}>
                {selectedOrg ? t("organization selected") : t("no organization")}
              </ProductStatusBadge>
            </div>
          </div>

          {errorMessage ? (
            <Alert variant="destructive">
              <AlertDescription>{errorMessage}</AlertDescription>
            </Alert>
          ) : null}
      </ProductHero>

      <Card className="card-enter">
        <CardHeader>
          <div className="flex items-center gap-3">
            <div className="flex size-10 items-center justify-center rounded-2xl bg-primary/10 text-primary">
              <FolderGit2 className="size-5" />
            </div>
            <div>
              <CardTitle>{t("Repository List")}</CardTitle>
              <CardDescription>
                {selectedOrg
                  ? t("Repositories under the selected organization.")
                  : t("Select an organization to view repositories.")}
              </CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent className="grid gap-3">
          {isLoading ? <p className="text-sm text-muted-foreground">{t("Loading repositories...")}</p> : null}

          {repos.map((repo) => (
            <div key={repo.id} className="group space-y-4 rounded-2xl border border-border/70 bg-background/55 p-4 transition-all duration-300 hover:-translate-y-0.5 hover:bg-background/75 hover:shadow-[0_22px_70px_-55px_hsl(var(--foreground)/0.65)]">
              <div className="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
                <div className="min-w-0 space-y-1">
                  <p className="truncate font-medium">
                    <Link
                      to={`/app/repositories/${repo.organization_id}/${repo.id}`}
                      className="underline-offset-4 hover:underline"
                    >
                      {repo.name}
                    </Link>
                  </p>
                  <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
                    <span>{repo.key}</span>
                    <span>/</span>
                    <span className="inline-flex items-center gap-1">
                      <GitBranch className="size-3" />
                      {repo.default_branch}
                    </span>
                  </div>
                  {repo.description ? (
                    <p className="text-sm text-muted-foreground">{repo.description}</p>
                  ) : null}
                </div>
                <Badge variant="secondary">{repo.visibility}</Badge>
              </div>

              <div className="rounded-2xl border bg-muted/40 px-3 py-2">
                <p className="truncate text-xs text-muted-foreground">{t("Clone URL")}</p>
                <p className="break-all font-mono text-xs">{repo.clone_http_url}</p>
              </div>

              <div className="flex flex-wrap items-center gap-2">
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  className="action-pop"
                  asChild
                >
                  <Link to={`/app/repositories/${repo.organization_id}/${repo.id}`}>
                    <FolderGit2 className="size-4" />
                    {t("Open")}
                  </Link>
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  className="action-pop"
                  onClick={() => copyCloneUrl(repo.clone_http_url)}
                >
                  <Copy className="size-4" />
                  {t("Copy Clone URL")}
                </Button>
                <Button
                  type="button"
                  variant="destructive"
                  size="sm"
                  className="action-pop"
                  disabled={isDeleting}
                  onClick={() => submitDelete(repo)}
                >
                  <Trash2 className="size-4" />
                  {t("Delete")}
                </Button>
              </div>
            </div>
          ))}

          {repos.length === 0 && !errorMessage && !isLoading ? (
            <p className="text-sm text-muted-foreground">{t("No repositories available.")}</p>
          ) : null}
        </CardContent>
      </Card>

      <Modal open={isCreateModalOpen} onClose={() => setCreateModalOpen(false)} title={t("Create Repository")}>
        <form className="grid gap-3 md:grid-cols-2" onSubmit={submitCreate}>
          <div className="space-y-2 md:col-span-2">
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
          <div className="space-y-2">
            <Label htmlFor="repo-key">{t("Repository key")}</Label>
            <Input
              id="repo-key"
              placeholder={t("Repository key")}
              value={key}
              onChange={(event) => setKey(event.target.value)}
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="repo-name">{t("Repository name")}</Label>
            <Input
              id="repo-name"
              placeholder={t("Repository Name")}
              value={name}
              onChange={(event) => setName(event.target.value)}
              required
            />
          </div>
          <div className="space-y-2 md:col-span-2">
            <Label htmlFor="repo-description">{t("Description")}</Label>
            <Input
              id="repo-description"
              placeholder={t("Description (optional)")}
              value={description}
              onChange={(event) => setDescription(event.target.value)}
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="repo-default-branch">{t("Default branch")}</Label>
            <Input
              id="repo-default-branch"
              placeholder={t("Default branch")}
              value={defaultBranch}
              onChange={(event) => setDefaultBranch(event.target.value)}
            />
          </div>
          <div className="space-y-2">
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
          <div className="space-y-2">
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
          <div className="space-y-2">
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
