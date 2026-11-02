import { Link, useNavigate } from "react-router";
import { useList, useOne } from "@refinedev/core";
import { ArrowRight, Building2, FolderGit2, GitBranch, Inbox, ShieldCheck } from "lucide-react";
import { useI18n } from "@/lib/i18n";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { ActivityFeed } from "@/components/activity/activity-feed";
import { DataList, DataRow } from "@/components/surface/data-list";
import { StatCard } from "@/components/surface/stat-card";
import { Section } from "@/components/surface/section";
import type { OrganizationView, RepositoryView, UserView } from "@/pages/types";
import { useActivity } from "./use-activity";
import { useTodoList } from "./use-todos";

export function AppDashboardPage(): JSX.Element {
  const { t } = useI18n();
  const navigate = useNavigate();
  const userQuery = useOne<UserView>({ resource: "profile", id: "me" });
  const orgQuery = useList<OrganizationView>({ resource: "organizations" });
  const repoQuery = useList<RepositoryView>({ resource: "my-projects" });
  const todoQuery = useTodoList("pending");
  const activity = useActivity(8);

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
  const activityError = activity.query.error instanceof Error ? activity.query.error.message : null;

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
      <section className="overflow-hidden rounded-md border border-border bg-card" aria-labelledby="dashboard-title">
        <div className="flex flex-col gap-4 border-b border-border px-4 py-4 sm:px-5 lg:flex-row lg:items-center lg:justify-between">
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
        <div className="grid gap-2 px-4 py-3 sm:grid-cols-2 sm:px-5 xl:grid-cols-4">
          {statusItems.map((item) => (
            <StatCard key={item.label} {...item} />
          ))}
        </div>
      </section>

      {errorMessage ? <Alert variant="destructive"><AlertDescription>{errorMessage}</AlertDescription></Alert> : null}

      <div className="grid gap-4 xl:grid-cols-[minmax(0,1.45fr)_minmax(280px,0.75fr)]">
        <Section
          title={t("Recent Projects")}
          description={t("Fast access to the repositories you can operate.")}
          flush
          actions={
            <Button variant="ghost" size="sm" onClick={() => navigate("/app/projects")}>
              {t("View all")}
              <ArrowRight aria-hidden="true" />
            </Button>
          }
        >
          {isLoading ? <ProjectListSkeleton /> : recentRepos.length === 0 ? (
            <div className="px-4 py-8 text-sm text-muted-foreground">{t("No projects available.")}</div>
          ) : (
            <DataList>
              {recentRepos.map((repo) => (
                <DataRow
                  key={repo.id}
                  onClick={() => navigate(`/app/projects/${repo.organization_id}/${repo.id}`)}
                >
                  <span className="flex min-w-0 flex-1 items-center gap-3">
                    <FolderGit2 className="size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
                    <span className="min-w-0">
                      <span className="block truncate text-sm font-medium">{repo.name}</span>
                      <span className="mt-0.5 flex items-center gap-1.5 truncate text-xs text-muted-foreground">
                        <GitBranch className="size-3" aria-hidden="true" />{repo.key} / {repo.default_branch}
                      </span>
                    </span>
                  </span>
                  <Badge variant="secondary" className="shrink-0">{repo.visibility}</Badge>
                </DataRow>
              ))}
            </DataList>
          )}
        </Section>

        <Section title={t("My Organizations")} description={t("Organizations you can operate in.")} flush>
          {isLoading ? <OrganizationListSkeleton /> : orgs.length === 0 ? (
            <div className="px-4 py-8 text-sm text-muted-foreground">{t("No organizations available.")}</div>
          ) : (
            <DataList>
              {orgs.map((org) => (
                <DataRow key={org.id}>
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm font-medium">{org.name}</p>
                    <p className="truncate font-mono text-xs text-muted-foreground">{org.key}</p>
                  </div>
                  <Badge variant="outline">{org.role}</Badge>
                </DataRow>
              ))}
            </DataList>
          )}
        </Section>
      </div>

      <Section
        title={t("Recent activity")}
        description={t("Latest events across the projects you can access.")}
        flush
        actions={
          <Button asChild variant="ghost" size="sm">
            <Link to="/app/activity">
              {t("View all")}
              <ArrowRight aria-hidden="true" />
            </Link>
          </Button>
        }
      >
        <ActivityFeed
          items={activity.items}
          isLoading={activity.query.isLoading}
          error={activityError}
          emptyText={t("No recent activity yet.")}
          skeletonRows={4}
        />
      </Section>
    </div>
  );
}

function ProjectListSkeleton(): JSX.Element {
  return <div className="space-y-2 p-3">{[0, 1, 2].map((item) => <Skeleton key={item} className="h-12 w-full" />)}</div>;
}

function OrganizationListSkeleton(): JSX.Element {
  return <div className="space-y-2 p-3">{[0, 1, 2].map((item) => <Skeleton key={item} className="h-10 w-full" />)}</div>;
}
