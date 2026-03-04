import { Link } from "react-router-dom";
import { useList, useOne } from "@refinedev/core";
import { useI18n } from "@/lib/i18n";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
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

  return (
    <div className="space-y-6 page-enter">
      <div>
        <h2 className="text-2xl font-semibold tracking-tight">{t("Dashboard")}</h2>
        <p className="text-sm text-muted-foreground">
          {t("Overview of your account, organizations, and repository workspace.")}
        </p>
      </div>

      {errorMessage ? (
        <p className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
          {errorMessage}
        </p>
      ) : null}

      <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
        <Card className="card-enter">
          <CardHeader>
            <CardDescription>{t("Organizations")}</CardDescription>
            <CardTitle className="text-3xl">{isLoading ? "--" : orgs.length}</CardTitle>
          </CardHeader>
          <CardContent className="pt-0">
            <p className="text-xs text-muted-foreground">{t("Memberships available in your scope.")}</p>
          </CardContent>
        </Card>

        <Card className="card-enter">
          <CardHeader>
            <CardDescription>{t("Repositories")}</CardDescription>
            <CardTitle className="text-3xl">{isLoading ? "--" : repos.length}</CardTitle>
          </CardHeader>
          <CardContent className="pt-0">
            <p className="text-xs text-muted-foreground">{t("Total repositories visible to you.")}</p>
          </CardContent>
        </Card>

        <Card className="card-enter">
          <CardHeader>
            <CardDescription>{t("Account Status")}</CardDescription>
            <CardTitle className="text-lg">{user?.status ?? "--"}</CardTitle>
          </CardHeader>
          <CardContent className="pt-0">
            <div className="flex items-center gap-2">
              <Badge variant={user?.is_super_admin ? "default" : "secondary"}>
                {user?.is_super_admin ? t("Super Admin") : t("User")}
              </Badge>
              <span className="text-xs text-muted-foreground">{t("Access role")}</span>
            </div>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-4 lg:grid-cols-2">
        <Card className="card-enter">
          <CardHeader>
            <CardTitle>{t("My Account")}</CardTitle>
            <CardDescription>{t("Identity and quick navigation.")}</CardDescription>
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

            <div className="flex flex-wrap gap-2">
              <Button asChild size="sm" className="action-pop">
                <Link to="/app/repositories">{t("Open Repositories")}</Link>
              </Button>
              <Button asChild size="sm" variant="outline" className="action-pop">
                <Link to="/app/profile">{t("Edit Profile")}</Link>
              </Button>
              {user?.is_super_admin ? (
                <Button asChild size="sm" variant="secondary" className="action-pop">
                  <Link to="/admin">{t("Go Admin")}</Link>
                </Button>
              ) : null}
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
                  <div key={org.id} className="flex items-center justify-between rounded-md border px-3 py-2">
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
