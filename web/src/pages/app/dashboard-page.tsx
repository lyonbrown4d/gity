import { Link } from "react-router-dom";
import { useList, useOne } from "@refinedev/core";
import { Activity, Building2, FolderGit2, ShieldCheck } from "lucide-react";
import { useI18n } from "@/lib/i18n";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { ProductHero, ProductMetricCard } from "@/components/ui/product";
import { Skeleton } from "@/components/ui/skeleton";
import type { OrganizationView, RepositoryView, UserView } from "@/pages/types";

export function AppDashboardPage(): JSX.Element {
  const { t } = useI18n();
  const userQuery = useOne<UserView>({
    resource: "profile",
    id: "me",
  });
  const orgQuery = useList<OrganizationView>({
    resource: "organizations",
  });
  const repoQuery = useList<RepositoryView>({
    resource: "my-repositories",
  });

  const user = userQuery.data?.data ?? null;
  const orgs = orgQuery.data?.data ?? [];
  const repos = repoQuery.data?.data ?? [];
  const isLoading = userQuery.isLoading || orgQuery.isLoading || repoQuery.isLoading;
  const errorMessage = userQuery.error instanceof Error
    ? userQuery.error.message
    : orgQuery.error instanceof Error
      ? orgQuery.error.message
      : repoQuery.error instanceof Error
        ? repoQuery.error.message
        : null;
  const roleLabel = user?.is_super_admin ? t("Super Admin") : t("User");

  return (
    <div className="space-y-6 page-enter">
      <ProductHero
        eyebrow={t("User Workspace")}
        title={t("Dashboard")}
        description={t("Overview of your account, organizations, and repository workspace.")}
        actions={(
          <>
            <Button asChild className="action-pop">
              <Link to="/app/repositories">{t("Open Repositories")}</Link>
            </Button>
            <Button asChild variant="outline" className="action-pop">
              <Link to="/app/profile">{t("Edit Profile")}</Link>
            </Button>
            {user?.is_super_admin ? (
              <Button asChild variant="secondary" className="action-pop">
                <Link to="/admin">{t("Go Admin")}</Link>
              </Button>
            ) : null}
          </>
        )}
        aside={(
          <Card className="border-border/70 bg-background/60 shadow-none">
            <CardContent className="p-4">
            <p className="gity-eyebrow">{t("My Account")}</p>
            <div className="mt-4 space-y-3">
              <div className="flex items-center justify-between gap-3">
                <span className="text-sm text-muted-foreground">{t("Username")}</span>
                <span className="truncate text-sm font-semibold">{isLoading ? "--" : user?.username ?? "--"}</span>
              </div>
              <div className="flex items-center justify-between gap-3">
                <span className="text-sm text-muted-foreground">{t("Email")}</span>
                <span className="truncate text-sm font-semibold">{isLoading ? "--" : user?.email ?? "--"}</span>
              </div>
              <div className="flex items-center justify-between gap-3">
                <span className="text-sm text-muted-foreground">{t("Access role")}</span>
                <Badge variant={user?.is_super_admin ? "default" : "secondary"}>{roleLabel}</Badge>
              </div>
            </div>
            </CardContent>
          </Card>
        )}
      />

      {errorMessage ? (
        <Alert variant="destructive">
          <AlertDescription>{errorMessage}</AlertDescription>
        </Alert>
      ) : null}

      <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
        <ProductMetricCard
          icon={Building2}
          label={t("Organizations")}
          value={isLoading ? "--" : String(orgs.length)}
          description={t("Memberships available in your scope.")}
        />
        <ProductMetricCard
          icon={FolderGit2}
          label={t("Repositories")}
          value={isLoading ? "--" : String(repos.length)}
          description={t("Total repositories visible to you.")}
        />
        <ProductMetricCard
          icon={ShieldCheck}
          label={t("Account Status")}
          value={user?.status ?? "--"}
          description={roleLabel}
        />
      </div>

      <div className="grid gap-4 lg:grid-cols-2">
        <Card className="overflow-hidden card-enter">
          <CardHeader>
            <div className="flex items-center gap-3">
              <div className="flex size-10 items-center justify-center rounded-2xl bg-primary/10 text-primary">
                <Activity className="size-5" />
              </div>
              <div>
                <CardTitle>{t("My Account")}</CardTitle>
                <CardDescription>{t("Identity and quick navigation.")}</CardDescription>
              </div>
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            {isLoading ? (
              <div className="space-y-2">
                <Skeleton className="h-5 w-44" />
                <Skeleton className="h-4 w-64" />
              </div>
            ) : (
              <div className="space-y-1">
                <p className="font-medium">{user?.username ?? "--"}</p>
                <p className="text-sm text-muted-foreground">{user?.email ?? "--"}</p>
              </div>
            )}

            <div className="rounded-2xl border bg-muted/35 p-3 text-xs text-muted-foreground">
              {t("Current account information and access status.")}
            </div>
          </CardContent>
        </Card>

        <Card className="card-enter">
          <CardHeader>
            <CardTitle>{t("My Organizations")}</CardTitle>
            <CardDescription>{t("Organizations you can operate in.")}</CardDescription>
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
                  <div key={org.id} className="flex items-center justify-between rounded-2xl border bg-background/55 px-3 py-3">
                    <div className="min-w-0">
                      <p className="truncate text-sm font-medium">{org.name}</p>
                      <p className="truncate text-xs text-muted-foreground">{org.key}</p>
                    </div>
                    <Badge variant="secondary">{org.role}</Badge>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
