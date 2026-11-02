import dayjs from "dayjs";
import { ArrowUpRight, Check, CircleCheckBig, Inbox } from "lucide-react";
import { Link, useSearchParams } from "react-router";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { DataList, DataRow } from "@/components/surface/data-list";
import { Section } from "@/components/surface/section";
import { Toolbar } from "@/components/surface/toolbar";
import { useI18n } from "@/lib/i18n";
import type { TodoState, TodoView } from "./todo-types";
import { useMarkTodoDone, useTodoList } from "./use-todos";

export function AppTodosPage(): JSX.Element {
  const { t } = useI18n();
  const [searchParams, setSearchParams] = useSearchParams();
  const state: TodoState = searchParams.get("state") === "done" ? "done" : "pending";
  const { todos, query } = useTodoList(state);
  const markTodo = useMarkTodoDone();
  const queryError = query.error instanceof Error ? query.error.message : null;

  return (
    <div className="flex flex-col gap-4 page-enter">
      <section aria-labelledby="todos-title" className="overflow-hidden rounded-md border border-border bg-card">
        <div className="flex flex-col gap-3 px-4 py-4 sm:flex-row sm:items-center sm:justify-between">
          <div className="min-w-0">
            <div className="mb-1 flex items-center gap-2 text-xs font-medium text-muted-foreground">
              <span className="size-2 rounded-full bg-primary" aria-hidden="true" />
              {t("Personal queue")}
            </div>
            <h1 id="todos-title" className="text-lg font-semibold tracking-tight">{t("Inbox")}</h1>
            <p className="mt-1 max-w-2xl text-sm text-muted-foreground">
              {t("Review project activity that needs your attention, then clear it from the queue.")}
            </p>
          </div>
          <div className="flex shrink-0 items-center gap-3 sm:text-right">
            <div>
              <p className="text-xs text-muted-foreground">{t(state === "pending" ? "Pending" : "Done")}</p>
              <p className="text-xl font-semibold leading-tight tabular-nums">{query.isLoading ? "--" : todos.length}</p>
              <p className="text-[11px] text-muted-foreground">{t("Items in this view")}</p>
            </div>
          </div>
        </div>
        <div className="flex flex-wrap items-center justify-between gap-3 border-t border-border px-4 py-2">
          <Toolbar>
            <Tabs
              value={state}
              onValueChange={(value) => {
                const next = new URLSearchParams(searchParams);
                next.set("state", value === "done" ? "done" : "pending");
                setSearchParams(next, { replace: true });
              }}
            >
              <TabsList aria-label={t("Todo state")}>
                <TabsTrigger value="pending">{t("Pending")}</TabsTrigger>
                <TabsTrigger value="done">{t("Done")}</TabsTrigger>
              </TabsList>
            </Tabs>
          </Toolbar>
          <Button type="button" variant="outline" size="sm" onClick={() => void query.refetch()} disabled={query.isFetching}>
            {query.isFetching ? t("Refreshing...") : t("Refresh")}
          </Button>
        </div>
      </section>

      {queryError ? (
        <Alert variant="destructive">
          <AlertDescription className="flex flex-wrap items-center justify-between gap-3">
            <span>{queryError}</span>
            <Button type="button" variant="outline" size="sm" onClick={() => void query.refetch()}>
              {t("Try again")}
            </Button>
          </AlertDescription>
        </Alert>
      ) : null}
      {markTodo.error ? (
        <Alert variant="destructive">
          <AlertDescription>{markTodo.error}</AlertDescription>
        </Alert>
      ) : null}

      <Section flush>
        {query.isLoading ? <TodoLoading /> : null}
        {!query.isLoading && !queryError && todos.length === 0 ? <TodoEmpty state={state} /> : null}
        {!query.isLoading && !queryError && todos.length > 0 ? (
          <DataList>
            {todos.map((todo) => (
              <TodoRow
                key={todo.id}
                todo={todo}
                isMarkingDone={markTodo.isPending && markTodo.activeTodoId === todo.id}
                onMarkDone={() => void markTodo.markDone(todo.id)}
              />
            ))}
          </DataList>
        ) : null}
      </Section>
    </div>
  );
}

function TodoRow({
  todo,
  isMarkingDone,
  onMarkDone,
}: {
  todo: TodoView;
  isMarkingDone: boolean;
  onMarkDone: () => void;
}): JSX.Element {
  const { t } = useI18n();
  const actionUrl = resolveActionUrl(todo);

  return (
    <DataRow className="items-start gap-3 sm:items-center">
      <span className="mt-0.5 flex size-7 shrink-0 items-center justify-center rounded-md bg-primary/10 text-primary sm:mt-0">
        {todo.state === "done" ? <CircleCheckBig className="size-4" aria-hidden="true" /> : <Inbox className="size-4" aria-hidden="true" />}
      </span>
      <div className="min-w-0 flex-1">
        <div className="flex min-w-0 flex-wrap items-center gap-2">
          <p className="truncate text-[13px] font-medium">{todo.title}</p>
          <Badge variant={todo.state === "done" ? "secondary" : "default"} className="shrink-0">{t(todo.state)}</Badge>
        </div>
        <p className="mt-0.5 line-clamp-1 text-xs text-muted-foreground">{todo.summary}</p>
        <div className="mt-1 flex flex-wrap items-center gap-2 text-[11px] text-muted-foreground lg:hidden">
          <Badge variant="outline">{formatKind(todo.kind)}</Badge>
          <span>{dayjs(todo.created_at).format("MMM D, YYYY h:mm A")}</span>
        </div>
      </div>
      <div className="hidden shrink-0 items-center gap-2 text-[11px] text-muted-foreground lg:flex">
        <Badge variant="outline">{formatKind(todo.kind)}</Badge>
        <span className="tabular-nums">{dayjs(todo.created_at).format("MMM D, YYYY h:mm A")}</span>
      </div>
      <div className="flex shrink-0 items-center gap-1">
        <Button asChild variant="ghost" size="sm">
          <Link to={actionUrl}>
            {t("Open")}
            <ArrowUpRight className="size-4" />
          </Link>
        </Button>
        {todo.state === "pending" ? (
          <Button type="button" size="sm" onClick={onMarkDone} disabled={isMarkingDone}>
            <Check className="size-4" />
            {isMarkingDone ? t("Marking done...") : t("Mark done")}
          </Button>
        ) : null}
      </div>
    </DataRow>
  );
}

function TodoLoading(): JSX.Element {
  return (
    <div className="divide-y divide-border" aria-label="Loading todos">
      {[0, 1, 2].map((item) => (
        <div key={item} className="px-4 py-3"><Skeleton className="h-8 w-full" /></div>
      ))}
    </div>
  );
}

function TodoEmpty({ state }: { state: TodoState }): JSX.Element {
  const { t } = useI18n();
  return (
    <div className="flex min-h-48 flex-col items-center justify-center gap-2 px-6 py-8 text-center">
      <span className="flex size-9 items-center justify-center rounded-md bg-primary/10 text-primary">
        <CircleCheckBig className="size-5" aria-hidden="true" />
      </span>
      <p className="text-sm font-medium">{t(state === "pending" ? "You're all caught up." : "No completed todos yet.")}</p>
      <p className="max-w-md text-xs text-muted-foreground">
        {t(state === "pending" ? "New project activity will appear here." : "Completed items remain available for reference.")}
      </p>
    </div>
  );
}

const formatKind = (kind: string): string => kind.split("_").map((part) => part[0]?.toUpperCase() + part.slice(1)).join(" ");

const resolveActionUrl = (todo: TodoView): string =>
  todo.action_url.startsWith("/app/")
    ? todo.action_url
    : `/app/projects/${todo.organization_id}/${todo.project_id}`;
