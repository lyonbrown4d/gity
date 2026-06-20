import { Link } from "react-router-dom";
import { useList, useOne } from "@refinedev/core";
import { ArrowRight, Building2, FolderGit2, GitBranch, ShieldCheck } from "lucide-react";
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
    resource: "my-projects",
  });

  const user = userQuery.result ?? null;
  const orgs = orgQuery.result.data ?? [];
  const repos = repoQuery.result.data ?? [];
  const recentRepos = repos.slice(0, 5);
  const isLoading = userQuery.query.isLoading || orgQuery.query.isLoading || repoQuery.query.isLoading;
  const errorMessage = userQuery.query.error instanceof Error
    ? userQuery.query.error.message
    : orgQuery.query.error instanceof Error
      ? orgQuery.query.error.message
      : repoQuery.query.error instanceof Error
        ? repoQuery.query.error.message
        : null;
  const roleLabel = user?.is_super_admin ? t("Super Admin") : t("User");

  return (
    <div className="flex flex-col gap-5 page-enter">
      <ProductHero
        title={t("Dashboard")}
        description={t("Project activity, organization scope, and account access in one workspace.")}
        actions={(
          <>
            <Button asChild className="action-pop">
              <Link to="/app/projects">{t("Open Projects")}</Link>
            </Button>
            <Button asChild variant="outline" className="action-pop">
              <Link to="/app/profile">{t("Edit Profile")}</Link>
            </Button>
          </>
        )}
        aside={(
          <div className="rounded-lg border border-border/80 bg-background/70 p-4">
            <p className="gity-eyebrow">{t("My Account")}</p>
            <div className="mt-4 flex flex-col gap-3">
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
          </div>
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
          label={t("Projects")}
          value={isLoading ? "--" : String(repos.length)}
          description={t("Total projects visible to you.")}
        />
        <ProductMetricCard
          icon={ShieldCheck}
          label={t("Account Status")}
          value={user?.status ?? "--"}
          description={roleLabel}
        />
      </div>

      <div className="grid gap-4 xl:grid-cols-[1.35fr_0.85fr]">
        <Card className="overflow-hidden card-enter">
          <CardHeader>
            <div className="flex items-center justify-between gap-3">
              <div>
                <CardTitle>{t("Recent Projects")}</CardTitle>
                <CardDescription>{t("Fast access to the repositories you can operate.")}</CardDescription>
              </div>
              <Button variant="outline" size="sm" asChild>
                <Link to="/app/projects">
                  {t("View all")}
                  <ArrowRight className="size-4" />
                </Link>
              </Button>
            </div>
          </CardHeader>
          <CardContent>
            {isLoading ? (
              <div className="flex flex-col gap-2">
                <Skeleton className="h-12 w-full" />
                <Skeleton className="h-12 w-full" />
                <Skeleton className="h-12 w-full" />
              </div>
            ) : recentRepos.length === 0 ? (
              <p className="text-sm text-muted-foreground">{t("No projects available.")}</p>
            ) : (
              <div className="overflow-hidden rounded-lg border">
                {recentRepos.map((repo) => (
                  <Link
                    key={repo.id}
                    to={`/app/projects/${repo.organization_id}/${repo.id}`}
                    className="flex items-center justify-between gap-4 border-b px-3 py-3 text-sm transition-colors last:border-b-0 hover:bg-muted/60 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  >
                    <span className="min-w-0">
                      <span className="block truncate font-medium">{repo.name}</span>
                      <span className="mt-1 flex items-center gap-2 truncate text-xs text-muted-foreground">
                        <GitBranch className="size-3" />
                        {repo.key} / {repo.default_branch}
                      </span>
                    </span>
                    <Badge variant="secondary" className="shrink-0">
                      {repo.visibility}
                    </Badge>
                  </Link>
                ))}
              </div>
            )}
          </CardContent>
        </Card>

        <Card className="card-enter">
          <CardHeader>
            <CardTitle>{t("My Organizations")}</CardTitle>
            <CardDescription>{t("Organizations you can operate in.")}</CardDescription>
          </CardHeader>
          <CardContent>
            {isLoading ? (
              <div className="flex flex-col gap-2">
                <Skeleton className="h-10 w-full" />
                <Skeleton className="h-10 w-full" />
                <Skeleton className="h-10 w-full" />
              </div>
            ) : orgs.length === 0 ? (
              <p className="text-sm text-muted-foreground">{t("No organizations available.")}</p>
            ) : (
              <div className="flex flex-col gap-2">
                {orgs.map((org) => (
                  <div key={org.id} className="flex items-center justify-between rounded-lg border bg-background px-3 py-3">
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
