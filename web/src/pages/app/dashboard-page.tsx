import { useMemo, useState } from "react";
import { Link, useSearchParams } from "react-router";
import { useList, useOne } from "@refinedev/core";
import { ArrowRight, GitBranch, Plus, Search } from "lucide-react";
import { useI18n } from "@/lib/i18n";
import { activityIcon } from "@/components/activity/activity-feed";
import { Section } from "@/components/surface/section";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Skeleton } from "@/components/ui/skeleton";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { formatRelativeTime } from "./repository/issues-utils";
import type { RepositoryView, UserView } from "@/pages/types";
import { useActivity } from "./use-activity";
import { useTodoList } from "./use-todos";
import { WorkList, WorkPreview } from "./dashboard/dashboard-work";

type WorkFilter = "all" | "reviews" | "assigned";

export const AppDashboardPage = (): JSX.Element => {
  const { locale, t } = useI18n();
  const [params, setParams] = useSearchParams();
  const requestedFilter = params.get("filter");
  const filter: WorkFilter = requestedFilter === "reviews" || requestedFilter === "assigned" ? requestedFilter : "all";
  const user = useOne<UserView>({ resource: "profile", id: "me" });
  const reposQuery = useList<RepositoryView>({ resource: "my-projects", pagination: { mode: "off" } });
  const todosQuery = useTodoList("pending");
  const activity = useActivity(4);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [search, setSearch] = useState("");
  const repos = reposQuery.result.data ?? [];
  const todos = todosQuery.todos;
  const filteredTodos = useMemo(() => todos.filter((todo) => {
    if (filter === "reviews" && todo.kind !== "merge_request_review_requested") return false;
    if (filter === "assigned" && todo.kind !== "issue_assigned" && todo.kind !== "merge_request_assigned") return false;
    const repo = repos.find((item) => item.id === todo.project_id);
    return `${todo.title} ${todo.summary} ${repo?.name ?? ""}`.toLowerCase().includes(search.toLowerCase());
  }), [todos, repos, filter, search]);
  const filteredRepos = repos.filter((repo) => `${repo.name} ${repo.key} ${repo.description ?? ""}`.toLowerCase().includes(search.toLowerCase()));
  const selected = filteredTodos.find((todo) => todo.id === selectedId) ?? filteredTodos[0] ?? null;
  const reviewCount = todos.filter((todo) => todo.kind === "merge_request_review_requested").length;
  const assignedCount = todos.filter((todo) => todo.kind === "issue_assigned" || todo.kind === "merge_request_assigned").length;
  const repoError = reposQuery.query.error instanceof Error ? reposQuery.query.error.message : null;
  const todoError = todosQuery.query.error instanceof Error ? todosQuery.query.error.message : null;
  const date = new Intl.DateTimeFormat(locale === "zh" ? "zh-CN" : "en-US", { dateStyle: "full" }).format(new Date());

  const setFilter = (value: string) => {
    if (value !== "all" && value !== "reviews" && value !== "assigned") return;
    const next = new URLSearchParams(params);
    if (value === "all") next.delete("filter");
    else next.set("filter", value);
    next.delete("view");
    setParams(next, { replace: true });
  };

  return (
    <div className="flex flex-col gap-5 page-enter">
      <header className="flex flex-wrap items-end justify-between gap-4">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">{t("Workspace")}</h1>
          <p className="mt-1 text-sm text-muted-foreground"><time dateTime={new Date().toISOString()}>{date}</time></p>
        </div>
        <Button asChild><Link to="/app/projects?create=1"><Plus aria-hidden="true" />{t("New Project")}</Link></Button>
      </header>

      {(repoError || todoError) ? <Alert variant="destructive"><AlertDescription className="flex flex-wrap items-center justify-between gap-3">
        <span>{repoError ? t("Projects could not be loaded.") : t("Your work could not be loaded.")} {repoError ?? todoError}</span>
        <Button variant="outline" size="sm" onClick={() => { void reposQuery.query.refetch(); void todosQuery.query.refetch(); }}>{t("Try again")}</Button>
      </AlertDescription></Alert> : null}

      <div className="relative w-full sm:w-80">
        <Search className="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" aria-hidden="true" />
        <Input value={search} onChange={(event) => setSearch(event.target.value)} placeholder={t("Search your work and projects")} aria-label={t("Search your work and projects")} className="pl-9" />
      </div>

      <div className="grid items-start gap-4 lg:grid-cols-[minmax(0,2fr)_minmax(280px,1fr)]">
        <Section title={t("Needs your attention")} flush actions={<Link className="text-sm font-medium text-primary hover:underline" to="/app/todos">{t("Open inbox")}<ArrowRight className="ml-1 inline size-4" /></Link>}>
          <Tabs value={filter} onValueChange={setFilter}>
            <TabsList aria-label={t("Filter work")} className="h-11 w-full justify-start gap-2 rounded-none border-b border-border bg-transparent p-0">
              <WorkFilterTab value="all" count={todos.length} label={t("All pending")} />
              <WorkFilterTab value="reviews" count={reviewCount} label={t("Review requests")} />
              <WorkFilterTab value="assigned" count={assignedCount} label={t("Assigned to me")} />
            </TabsList>
          </Tabs>
          {todosQuery.query.isLoading ? <WorkSkeleton /> : todoError ? <div className="px-4 py-8 text-sm text-muted-foreground">{t("Work is unavailable until the request succeeds.")}</div> : <WorkList items={filteredTodos} repos={repos} selectedId={selected?.id ?? null} onSelect={setSelectedId} />}
        </Section>
        <WorkPreview todo={selected} repo={repos.find((repo) => repo.id === selected?.project_id) ?? null} />
      </div>

      <div className="grid items-start gap-4 lg:grid-cols-[minmax(0,2fr)_minmax(280px,1fr)]">
        <Section title={t("Recent projects")} flush actions={<Link className="text-sm font-medium text-primary hover:underline" to="/app/projects">{t("View all")}<ArrowRight className="ml-1 inline size-4" /></Link>}>
          {reposQuery.query.isLoading ? <WorkSkeleton /> : repoError ? <p className="px-4 py-8 text-sm text-muted-foreground">{t("Projects are unavailable until the request succeeds.")}</p> : filteredRepos.length ? <div className="divide-y divide-border">{filteredRepos.slice(0, 12).map((repo) => <Link key={repo.id} to={`/app/projects/${repo.organization_id}/${repo.id}`} className="flex items-center gap-3 px-4 py-3 hover:bg-muted/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring">
            <span className="flex size-9 items-center justify-center rounded-md bg-primary/10 text-primary"><GitBranch className="size-4" aria-hidden="true" /></span><span className="min-w-0 flex-1"><span className="block truncate text-sm font-medium">{repo.name}</span><span className="block truncate text-xs text-muted-foreground">{repo.description || repo.full_path}</span></span><Badge variant="outline">{repo.default_branch}</Badge><ArrowRight className="size-4 text-muted-foreground" aria-hidden="true" />
          </Link>)}</div> : <p className="px-4 py-8 text-sm text-muted-foreground">{search ? t("No matching projects. Clear search to see all projects.") : t("No projects available.")}</p>}
        </Section>

        <Section title={t("Recent activity")} flush actions={<Button asChild variant="ghost" size="sm"><Link to="/app/activity">{t("View activity")}<ArrowRight aria-hidden="true" /></Link></Button>}>
          {activity.query.isLoading ? <WorkSkeleton /> : activity.query.error ? <p className="px-4 py-6 text-sm text-muted-foreground">{t("Recent activity could not be loaded.")}</p> : activity.items.length ? <div className="divide-y divide-border">{activity.items.map((item) => { const Icon = activityIcon(item.eventName); return <Link key={item.id || `${item.eventName}-${item.createdAt}`} to={item.organizationId && item.projectId ? `/app/projects/${item.organizationId}/${item.projectId}` : "/app/activity"} className="flex items-center gap-3 px-4 py-3 hover:bg-muted/50"><span className="flex size-8 items-center justify-center rounded-md bg-muted text-muted-foreground"><Icon className="size-4" aria-hidden="true" /></span><span className="min-w-0 flex-1"><span className="block truncate text-sm font-medium">{item.summary}</span><span className="block truncate text-xs text-muted-foreground">{item.actorName} · {item.projectName}</span></span><time className="text-xs text-muted-foreground">{formatRelativeTime(item.createdAt)}</time></Link>; })}</div> : <p className="px-4 py-6 text-sm text-muted-foreground">{t("No recent activity yet.")}</p>}
        </Section>
      </div>
    </div>
  );
};

const WorkFilterTab = ({ value, count, label }: { value: WorkFilter; count: number; label: string }): JSX.Element => (
  <TabsTrigger value={value} className="h-11 rounded-none border-b-2 border-transparent bg-transparent px-3 shadow-none data-[state=active]:border-primary data-[state=active]:bg-transparent data-[state=active]:text-foreground data-[state=active]:shadow-none">
    {label}<Badge variant="secondary" className="ml-1.5 px-1.5 py-0 text-[10px] tabular-nums">{count}</Badge>
  </TabsTrigger>
);

const WorkSkeleton = (): JSX.Element => <div className="space-y-3 p-4" aria-label="Loading"><Skeleton className="h-12 w-full" /><Skeleton className="h-12 w-full" /><Skeleton className="h-12 w-full" /></div>;
