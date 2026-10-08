import { Link } from "react-router";
import { useList, useOne } from "@refinedev/core";
import { ArrowRight, Building2, FolderGit2, GitBranch, Inbox, ShieldCheck } from "lucide-react";
import { useI18n } from "@/lib/i18n";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import type { OrganizationView, RepositoryView, UserView } from "@/pages/types";
import { useTodoList } from "./use-todos";

export function AppDashboardPage(): JSX.Element {
  const { t } = useI18n();
  const userQuery = useOne<UserView>({ resource: "profile", id: "me" });
  const orgQuery = useList<OrganizationView>({ resource: "organizations" });
  const repoQuery = useList<RepositoryView>({ resource: "my-projects" });
  const todoQuery = useTodoList("pending");

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
        : todoQuery.query.error instanceof Error
          ? todoQuery.query.error.message
          : null;
  const roleLabel = user?.is_super_admin ? t("Super Admin") : t("User");

  const statusItems = [
    {
      icon: Building2,
      label: t("Organizations"),
      value: isLoading ? "--" : String(orgs.length),
      detail: t("Memberships available in your scope."),
    },
    {
      icon: FolderGit2,
      label: t("Projects"),
      value: isLoading ? "--" : String(repos.length),
      detail: t("Total projects visible to you."),
    },
    {
      icon: Inbox,
      label: t("Pending Todos"),
      value: todoQuery.query.isLoading ? "--" : String(todoQuery.todos.length),
      detail: t("Project activity waiting in your inbox."),
    },
    {
      icon: ShieldCheck,
      label: t("Account Status"),
      value: user?.status ?? "--",
      detail: user ? `${roleLabel} / ${user.email}` : roleLabel,
    },
  ];

  return (
    <div className="flex flex-col gap-4 page-enter">
      <section className="overflow-hidden rounded-lg border border-border/80 bg-card shadow-sm" aria-labelledby="dashboard-title">
        <div className="flex flex-col gap-4 border-b border-border/70 px-4 py-4 sm:px-5 lg:flex-row lg:items-center lg:justify-between">
          <div className="min-w-0">
            <div className="mb-1 flex items-center gap-2 text-xs font-medium text-muted-foreground">
              <span className="size-2 rounded-full bg-primary" aria-hidden="true" />
              {t("Workspace")} / {user?.username ?? t("My Account")}
            </div>
            <h1 id="dashboard-title" className="text-xl font-semibold tracking-tight">{t("Dashboard")}</h1>
            <p className="mt-1 max-w-2xl text-sm text-muted-foreground">
              {t("Project activity, organization scope, and account access in one workspace.")}
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <Button asChild size="sm"><Link to="/app/projects">{t("Open Projects")}</Link></Button>
            <Button asChild size="sm" variant="outline"><Link to="/app/todos">{t("Open Inbox")}</Link></Button>
            <Button asChild size="sm" variant="outline"><Link to="/app/profile">{t("Edit Profile")}</Link></Button>
          </div>
        </div>
        <div className="grid divide-y divide-border/70 sm:grid-cols-2 sm:divide-x sm:divide-y-0 xl:grid-cols-4">
          {statusItems.map(({ icon: Icon, label, value, detail }) => (
            <div key={label} role="group" aria-label={label} className="flex items-center gap-3 px-4 py-3 sm:px-5">
              <span className="flex size-8 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground">
                <Icon className="size-4" aria-hidden="true" />
              </span>
              <div className="min-w-0">
                <p className="truncate text-xs text-muted-foreground">{label}</p>
                <p className="truncate text-sm font-semibold">{value}</p>
                <p className="mt-0.5 truncate text-[11px] text-muted-foreground" title={detail}>{detail}</p>
              </div>
            </div>
          ))}
        </div>
      </section>

      {errorMessage ? <Alert variant="destructive"><AlertDescription>{errorMessage}</AlertDescription></Alert> : null}

      <div className="grid gap-4 xl:grid-cols-[minmax(0,1.45fr)_minmax(280px,0.75fr)]">
        <Card className="overflow-hidden shadow-sm">
          <CardHeader className="flex-row items-center justify-between gap-3 space-y-0 border-b border-border/70 p-4 sm:px-5">
            <div>
              <CardTitle className="text-base">{t("Recent Projects")}</CardTitle>
              <CardDescription className="mt-1">{t("Fast access to the repositories you can operate.")}</CardDescription>
            </div>
            <Button variant="ghost" size="sm" asChild>
              <Link to="/app/projects">{t("View all")}<ArrowRight aria-hidden="true" /></Link>
            </Button>
          </CardHeader>
          <CardContent className="p-0">
            {isLoading ? <ProjectListSkeleton /> : recentRepos.length === 0 ? (
              <div className="px-5 py-8 text-sm text-muted-foreground">{t("No projects available.")}</div>
            ) : recentRepos.map((repo) => (
              <Link
                key={repo.id}
                to={`/app/projects/${repo.organization_id}/${repo.id}`}
                className="flex items-center justify-between gap-4 border-b border-border/70 px-4 py-3 text-sm transition-colors last:border-b-0 hover:bg-muted/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring sm:px-5"
              >
                <span className="flex min-w-0 items-center gap-3">
                  <FolderGit2 className="size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
                  <span className="min-w-0">
                    <span className="block truncate font-medium">{repo.name}</span>
                    <span className="mt-0.5 flex items-center gap-1.5 truncate text-xs text-muted-foreground">
                      <GitBranch className="size-3" aria-hidden="true" />{repo.key} / {repo.default_branch}
                    </span>
                  </span>
                </span>
                <Badge variant="secondary" className="shrink-0">{repo.visibility}</Badge>
              </Link>
            ))}
          </CardContent>
        </Card>

        <Card className="overflow-hidden shadow-sm">
          <CardHeader className="border-b border-border/70 p-4 sm:px-5">
            <CardTitle className="text-base">{t("My Organizations")}</CardTitle>
            <CardDescription>{t("Organizations you can operate in.")}</CardDescription>
          </CardHeader>
          <CardContent className="p-0">
            {isLoading ? <OrganizationListSkeleton /> : orgs.length === 0 ? (
              <div className="px-5 py-8 text-sm text-muted-foreground">{t("No organizations available.")}</div>
            ) : orgs.map((org) => (
              <div key={org.id} className="flex items-center justify-between gap-3 border-b border-border/70 px-4 py-3 last:border-b-0 sm:px-5">
                <div className="min-w-0">
                  <p className="truncate text-sm font-medium">{org.name}</p>
                  <p className="truncate font-mono text-xs text-muted-foreground">{org.key}</p>
                </div>
                <Badge variant="outline">{org.role}</Badge>
              </div>
            ))}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

function ProjectListSkeleton(): JSX.Element {
  return <div className="space-y-px p-3">{[0, 1, 2].map((item) => <Skeleton key={item} className="h-14 w-full" />)}</div>;
}

function OrganizationListSkeleton(): JSX.Element {
  return <div className="space-y-px p-3">{[0, 1, 2].map((item) => <Skeleton key={item} className="h-12 w-full" />)}</div>;
}
