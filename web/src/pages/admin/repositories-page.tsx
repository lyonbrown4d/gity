import { useEffect, useMemo, useState } from "react";
import { useDelete, useList } from "@refinedev/core";
import { useI18n } from "@/lib/i18n";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import type { OrganizationView, RepositoryView } from "@/pages/types";

export function AdminRepositoriesPage(): JSX.Element {
  const { t } = useI18n();
  const [selectedOrg, setSelectedOrg] = useState<string>("");
  const [actionError, setActionError] = useState<string | null>(null);

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

  const repoQuery = useList<RepositoryView>({
    resource: "repositories",
    meta: {
      organization_id: selectedOrg,
    },
    queryOptions: {
      enabled: Boolean(selectedOrg),
    },
  });
  const repos = repoQuery.data?.data ?? [];

  const selectedOrgName = useMemo(
    () => orgs.find((item) => item.id === selectedOrg)?.name ?? t("N/A"),
    [orgs, selectedOrg, t],
  );
  const isLoading = orgQuery.isLoading || repoQuery.isLoading;
  const errorMessage = actionError
    ?? (orgQuery.error instanceof Error
      ? orgQuery.error.message
      : repoQuery.error instanceof Error
        ? repoQuery.error.message
        : null);

  const submitDelete = (repo: RepositoryView) => {
    const confirmText = t("Delete repository \"{name}\"?").replace("{name}", repo.name);
    if (!window.confirm(confirmText)) {
      return;
    }
    setActionError(null);
    deleteRepository(
      {
        resource: "repositories",
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

  return (
    <div className="space-y-4 page-enter">
      <Card className="card-enter">
        <CardHeader>
          <CardTitle>{t("Repositories")}</CardTitle>
          <CardDescription>
            {t("Admin can view and delete repositories. New repository creation is handled in user workspace.")}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid gap-3 md:grid-cols-[240px_1fr] md:items-end">
            <div className="space-y-2">
              <Label htmlFor="admin-org-select">{t("Organization")}</Label>
              <Select value={selectedOrg} onValueChange={setSelectedOrg}>
                <SelectTrigger id="admin-org-select">
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
              <Badge variant="secondary">{selectedOrgName}</Badge>
              <Badge variant="outline">
                {repos.length} {t("repos")}
              </Badge>
            </div>
          </div>

          {errorMessage ? (
            <Alert variant="destructive">
              <AlertDescription>{errorMessage}</AlertDescription>
            </Alert>
          ) : null}
        </CardContent>
      </Card>

      <Card className="card-enter">
        <CardHeader>
          <CardTitle>{t("Repository List")}</CardTitle>
          <CardDescription>{t("Repositories under the selected organization.")}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          {isLoading ? <p className="text-sm text-muted-foreground">{t("Loading repositories...")}</p> : null}
          <div className="space-y-3">
            {repos.map((repo) => (
              <div key={repo.id} className="space-y-3 rounded-lg border p-4">
                <div className="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
                  <div className="min-w-0">
                    <p className="truncate font-medium">{repo.name}</p>
                    <p className="truncate text-xs text-muted-foreground">
                      {repo.key} · {t("default:")} {repo.default_branch}
                    </p>
                    <p className="truncate font-mono text-[11px] text-muted-foreground">
                      UUID: {repo.uuid}
                    </p>
                    {repo.description ? (
                      <p className="mt-1 text-sm text-muted-foreground">{repo.description}</p>
                    ) : null}
                  </div>
                  <Badge>{repo.visibility}</Badge>
                </div>

                <div className="rounded-md border bg-muted/40 px-3 py-2">
                  <p className="text-xs text-muted-foreground">{t("Clone URL")}</p>
                  <p className="break-all font-mono text-xs">{repo.clone_http_url}</p>
                </div>

                <div className="flex flex-wrap items-center gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    type="button"
                    className="action-pop"
                    onClick={() => copyCloneUrl(repo.clone_http_url)}
                  >
                    {t("Copy Clone URL")}
                  </Button>
                  <Button
                    variant="destructive"
                    size="sm"
                    type="button"
                    className="action-pop"
                    disabled={isDeleting}
                    onClick={() => submitDelete(repo)}
                  >
                    {t("Delete")}
                  </Button>
                </div>
              </div>
            ))}
            {repos.length === 0 && !errorMessage && !isLoading ? (
              <p className="text-sm text-muted-foreground">{t("No repositories in this organization.")}</p>
            ) : null}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
