import { useList } from "@refinedev/core";
import { useI18n } from "@/lib/i18n";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import type { OrganizationView, RepositoryView } from "@/pages/types";
import {
  ConsoleEmpty,
  ConsoleLoadingRows,
  ConsolePageHeader,
  ConsolePanel,
  MetricStrip,
  StatusDot,
} from "./workspace/console-primitives";

export function AdminOverviewPage(): JSX.Element {
  const { t } = useI18n();
  const orgQuery = useList<OrganizationView>({
    resource: "organizations",
  });
  const orgs = orgQuery.result.data ?? [];
  const primaryOrgId = orgs[0]?.id;

  const repoQuery = useList<RepositoryView>({
    resource: "projects",
    meta: {
      organization_id: primaryOrgId,
    },
    queryOptions: {
      enabled: Boolean(primaryOrgId),
    },
  });
  const repos = repoQuery.result.data ?? [];
  const isLoading = orgQuery.query.isLoading || repoQuery.query.isLoading;
  const errorMessage = orgQuery.query.error instanceof Error
    ? orgQuery.query.error.message
    : repoQuery.query.error instanceof Error
      ? repoQuery.query.error.message
      : null;

  return (
    <div className="space-y-4 page-enter">
      <ConsolePageHeader
        eyebrow={t("Admin Side")}
        title={t("Admin Overview")}
        description={t("Monitor organizations, projects, and current control-plane baseline.")}
      />

      {errorMessage ? (
        <Alert variant="destructive">
          <AlertDescription>{errorMessage}</AlertDescription>
        </Alert>
      ) : null}

      <MetricStrip items={[
        { label: t("Organizations"), value: isLoading ? "--" : orgs.length, detail: t("total"), active: orgs.length > 0 },
        { label: t("Projects"), value: isLoading ? "--" : repos.length, detail: t("projects"), active: repos.length > 0 },
        { label: t("Baseline"), value: "03", detail: t("System Composition"), active: !errorMessage },
      ]} />

      <div className="grid gap-4 xl:grid-cols-[minmax(0,1.5fr)_minmax(280px,0.7fr)]">
        <ConsolePanel
          title={t("Organization Snapshot")}
          description={t("Quick view of organizations and role mapping.")}
          meta={`${orgs.length} ${t("total")}`}
          bodyClassName="p-0"
        >
          {isLoading ? <div className="p-3"><ConsoleLoadingRows rows={4} /></div> : (
            <Table>
              <TableHeader>
                <TableRow className="bg-muted/20">
                  <TableHead className="h-8 px-3 text-[10px] uppercase tracking-wider">{t("Organization")}</TableHead>
                  <TableHead className="h-8 text-[10px] uppercase tracking-wider">{t("Role")}</TableHead>
                  <TableHead className="h-8 px-3 text-right text-[10px] uppercase tracking-wider">ID</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {orgs.map((org) => (
                  <TableRow key={org.id}>
                    <TableCell className="px-3 py-2">
                      <div className="flex items-center gap-2">
                        <StatusDot />
                        <div><p className="text-xs font-medium">{org.name}</p><p className="font-mono text-[10px] text-muted-foreground">{org.key}</p></div>
                      </div>
                    </TableCell>
                    <TableCell className="py-2"><Badge variant="outline">{org.role}</Badge></TableCell>
                    <TableCell className="max-w-44 truncate px-3 py-2 text-right font-mono text-[10px] text-muted-foreground">{org.id}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
          {!isLoading && orgs.length === 0 ? <ConsoleEmpty>{t("No organizations available.")}</ConsoleEmpty> : null}
        </ConsolePanel>

        <ConsolePanel title={t("System Composition")} description={t("Current baseline aligns route auth with role-scoped layouts and CRUD pages.")}>
          <div className="divide-y divide-border/70">
            {[
              ["refine", t("Refine-driven routing and auth flow")],
              ["shadcn/ui", t("Platform")],
              [t("split user/admin"), t("Access role")],
            ].map(([name, detail]) => (
              <div key={name} className="flex items-center gap-3 py-2 first:pt-0 last:pb-0">
                <StatusDot />
                <div className="min-w-0 flex-1"><p className="font-mono text-xs font-medium">{name}</p><p className="truncate text-[10px] text-muted-foreground">{detail}</p></div>
                <Badge variant="secondary" className="text-[9px]">{t("active")}</Badge>
              </div>
            ))}
          </div>
        </ConsolePanel>
      </div>
    </div>
  );
}
