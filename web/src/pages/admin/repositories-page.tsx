import { useEffect, useMemo, useState } from "react";
import { Copy, Trash2 } from "lucide-react";
import { useDelete, useList } from "@refinedev/core";
import { ConfirmAction } from "@/components/common/confirm-action";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { useI18n } from "@/lib/i18n";
import type { OrganizationView, RepositoryView } from "@/pages/types";
import { ConsoleEmpty, ConsoleLoadingRows, ConsolePageHeader, ConsolePanel, MetricStrip, StatusDot } from "./workspace/console-primitives";

export const AdminRepositoriesPage = (): JSX.Element => {
  const { t } = useI18n();
  const [selectedOrg, setSelectedOrg] = useState("");
  const [actionError, setActionError] = useState<string | null>(null);

  const { mutate: deleteRepository, mutation: { isPending: isDeleting } } = useDelete<RepositoryView>();

  const orgQuery = useList<OrganizationView>({ resource: "organizations" });
  const orgs = orgQuery.result.data ?? [];

  useEffect(() => {
    if (!selectedOrg && orgs[0]) setSelectedOrg(orgs[0].id);
  }, [orgs, selectedOrg]);

  const repoQuery = useList<RepositoryView>({
    resource: "projects",
    meta: { organization_id: selectedOrg },
    queryOptions: { enabled: Boolean(selectedOrg) },
  });
  const repos = repoQuery.result.data ?? [];

  const selectedOrgName = useMemo(
    () => orgs.find((item) => item.id === selectedOrg)?.name ?? t("N/A"),
    [orgs, selectedOrg, t],
  );
  const isLoading = orgQuery.query.isLoading || repoQuery.query.isLoading;
  const queryError = orgQuery.query.error ?? repoQuery.query.error;
  const errorMessage = actionError ?? (queryError instanceof Error ? queryError.message : null);

  const submitDelete = (repo: RepositoryView, confirmation: string) => {
    setActionError(null);
    deleteRepository(
      { resource: "projects", id: repo.id, meta: { confirmation } },
      {
        onSuccess: async () => repoQuery.query.refetch(),
        onError: (error) => setActionError(error instanceof Error ? error.message : t("Failed to delete project")),
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
      <ConsolePageHeader
        eyebrow={t("Admin Side")}
        title={t("Projects")}
        description={t("Admin can view and delete projects. New project creation is handled in user workspace.")}
      />

      {errorMessage ? <Alert variant="destructive"><AlertDescription>{errorMessage}</AlertDescription></Alert> : null}

      <section className="rounded-md border border-border/80 bg-card p-3">
        <div className="grid gap-3 sm:grid-cols-[minmax(220px,320px)_1fr] sm:items-end">
          <div className="space-y-1.5">
            <Label htmlFor="admin-org-select" className="font-mono text-[10px] uppercase tracking-wider text-muted-foreground">
              {t("Organization")}
            </Label>
            <Select value={selectedOrg} onValueChange={setSelectedOrg}>
              <SelectTrigger id="admin-org-select" className="h-8 text-xs"><SelectValue placeholder={t("Organization")} /></SelectTrigger>
              <SelectContent>
                {orgs.map((org) => <SelectItem key={org.id} value={org.id}>{org.name}</SelectItem>)}
              </SelectContent>
            </Select>
          </div>
          <div className="flex items-center gap-2 pb-0.5">
            <Badge variant="secondary">{selectedOrgName}</Badge>
            <Badge variant="outline">{repos.length} {t("projects")}</Badge>
          </div>
        </div>
      </section>

      <MetricStrip items={[
        { label: t("Organizations"), value: orgs.length, detail: t("total"), active: orgs.length > 0 },
        { label: t("Projects"), value: repos.length, detail: t("projects"), active: repos.length > 0 },
        { label: t("Organization"), value: selectedOrg ? "01" : "00", detail: selectedOrgName, active: Boolean(selectedOrg) },
      ]} />

      <ConsolePanel title={t("Project List")} description={t("Projects under the selected organization.")} meta={`${repos.length} ${t("projects")}`} bodyClassName="p-0">
        {isLoading ? <div className="p-3"><ConsoleLoadingRows rows={4} /></div> : (
          <Table>
            <TableHeader>
              <TableRow className="bg-muted/20">
                <TableHead className="h-8 px-3 text-[10px] uppercase tracking-wider">{t("Project")}</TableHead>
                <TableHead className="h-8 text-[10px] uppercase tracking-wider">{t("Clone URL")}</TableHead>
                <TableHead className="h-8 text-[10px] uppercase tracking-wider">{t("Visibility")}</TableHead>
                <TableHead className="h-8 px-3 text-right text-[10px] uppercase tracking-wider">{t("Actions")}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {repos.map((repo) => (
                <TableRow key={repo.id}>
                  <TableCell className="px-3 py-2">
                    <div className="flex items-start gap-2">
                      <span className="mt-1.5"><StatusDot active={repo.status === "active"} /></span>
                      <div className="min-w-0">
                        <p className="truncate text-xs font-medium">{repo.name}</p>
                        <p className="truncate font-mono text-[10px] text-muted-foreground">{repo.key} / {repo.default_branch}</p>
                        {repo.description ? <p className="mt-0.5 max-w-72 truncate text-[10px] text-muted-foreground">{repo.description}</p> : null}
                      </div>
                    </div>
                  </TableCell>
                  <TableCell className="max-w-72 py-2 font-mono text-[10px] text-muted-foreground">
                    <p className="truncate" title={repo.clone_http_url}>{repo.clone_http_url}</p>
                    <p className="truncate text-[9px] opacity-70">{repo.uuid}</p>
                  </TableCell>
                  <TableCell className="py-2"><Badge variant="outline">{repo.visibility}</Badge></TableCell>
                  <TableCell className="px-3 py-2">
                    <div className="flex justify-end gap-1">
                      <Button type="button" variant="ghost" size="icon" className="size-7" onClick={() => copyCloneUrl(repo.clone_http_url)} aria-label={`${t("Copy Clone URL")} ${repo.name}`} title={t("Copy Clone URL")}>
                        <Copy className="size-3.5" />
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
                        <Button type="button" variant="ghost" size="icon" className="size-7 text-destructive hover:text-destructive" disabled={isDeleting} aria-label={`${t("Delete")} ${repo.name}`} title={t("Delete")}>
                          <Trash2 className="size-3.5" />
                        </Button>
                      </ConfirmAction>
                    </div>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
        {!isLoading && repos.length === 0 && !errorMessage ? <ConsoleEmpty>{t("No projects in this organization.")}</ConsoleEmpty> : null}
      </ConsolePanel>
    </div>
  );
};
