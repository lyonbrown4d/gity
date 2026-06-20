import { useList } from "@refinedev/core";
import { useI18n } from "@/lib/i18n";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import type { OrganizationView, RepositoryView } from "@/pages/types";

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
    <div className="space-y-6 page-enter">
      <div>
        <h2 className="text-2xl font-semibold tracking-tight">{t("Admin Overview")}</h2>
        <p className="text-sm text-muted-foreground">
          {t("Monitor organizations, projects, and current control-plane baseline.")}
        </p>
      </div>

      {errorMessage ? (
        <Alert variant="destructive">
          <AlertDescription>{errorMessage}</AlertDescription>
        </Alert>
      ) : null}

      <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
        <Card className="card-enter">
          <CardHeader>
            <CardDescription>{t("Organizations")}</CardDescription>
            <CardTitle className="text-3xl">{isLoading ? "--" : orgs.length}</CardTitle>
          </CardHeader>
          <CardContent className="pt-0">
            <p className="text-xs text-muted-foreground">{t("Total organizations visible to admin.")}</p>
          </CardContent>
        </Card>

        <Card className="card-enter">
          <CardHeader>
            <CardDescription>{t("Projects")}</CardDescription>
            <CardTitle className="text-3xl">{isLoading ? "--" : repos.length}</CardTitle>
          </CardHeader>
          <CardContent className="pt-0">
            <p className="text-xs text-muted-foreground">{t("Projects under the primary organization.")}</p>
          </CardContent>
        </Card>

        <Card className="card-enter">
          <CardHeader>
            <CardDescription>{t("Baseline")}</CardDescription>
            <CardTitle className="text-lg">{t("System Composition")}</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 pt-0">
            <div className="flex flex-wrap gap-2">
              <Badge variant="secondary">refine</Badge>
              <Badge variant="secondary">shadcn/ui</Badge>
              <Badge variant="secondary">{t("split user/admin")}</Badge>
            </div>
            <p className="text-xs text-muted-foreground">{t("Current baseline aligns route auth with role-scoped layouts and CRUD pages.")}</p>
          </CardContent>
        </Card>
      </div>

      <Card className="card-enter">
        <CardHeader>
          <CardTitle>{t("Organization Snapshot")}</CardTitle>
          <CardDescription>
            {t("Quick view of organizations and role mapping.")}
          </CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="space-y-2">
              <Skeleton className="h-10 w-full" />
              <Skeleton className="h-10 w-full" />
              <Skeleton className="h-10 w-full" />
            </div>
          ) : orgs.length === 0 ? (
            <p className="text-sm text-muted-foreground">{t("No organizations available.")}</p>
          ) : (
            <div className="space-y-2">
              {orgs.map((org) => (
                <div key={org.id} className="flex items-center justify-between rounded-md border px-3 py-2">
                  <div className="min-w-0">
                    <p className="truncate text-sm font-medium">{org.name}</p>
                    <p className="truncate text-xs text-muted-foreground">{org.key}</p>
                  </div>
                  <Badge variant="outline">{org.role}</Badge>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
