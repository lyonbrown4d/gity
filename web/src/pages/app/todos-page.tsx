import { useState } from "react";
import dayjs from "dayjs";
import { ArrowUpRight, Check, CircleCheckBig, Inbox } from "lucide-react";
import { Link } from "react-router";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ProductHero } from "@/components/ui/product";
import { Skeleton } from "@/components/ui/skeleton";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useI18n } from "@/lib/i18n";
import type { TodoState, TodoView } from "./todo-types";
import { useMarkTodoDone, useTodoList } from "./use-todos";

export function AppTodosPage(): JSX.Element {
  const { t } = useI18n();
  const [state, setState] = useState<TodoState>("pending");
  const { todos, query } = useTodoList(state);
  const markTodo = useMarkTodoDone();
  const queryError = query.error instanceof Error ? query.error.message : null;

  return (
    <div className="flex flex-col gap-5 page-enter">
      <ProductHero
        eyebrow={t("Personal queue")}
        title={<h1>{t("Inbox")}</h1>}
        description={t("Review project activity that needs your attention, then clear it from the queue.")}
        aside={(
          <div className="rounded-xl border border-border/80 bg-background/70 p-4">
            <p className="gity-eyebrow">{t(state === "pending" ? "Pending" : "Done")}</p>
            <p className="mt-2 text-3xl font-semibold">{query.isLoading ? "--" : todos.length}</p>
            <p className="mt-1 text-xs text-muted-foreground">{t("Items in this view")}</p>
          </div>
        )}
      />

      <div className="flex flex-wrap items-center justify-between gap-3">
        <Tabs value={state} onValueChange={(value) => setState(value === "done" ? "done" : "pending")}>
          <TabsList aria-label={t("Todo state")}>
            <TabsTrigger value="pending">{t("Pending")}</TabsTrigger>
            <TabsTrigger value="done">{t("Done")}</TabsTrigger>
          </TabsList>
        </Tabs>
        <Button type="button" variant="outline" size="sm" onClick={() => void query.refetch()} disabled={query.isFetching}>
          {query.isFetching ? t("Refreshing...") : t("Refresh")}
        </Button>
      </div>

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

      {query.isLoading ? <TodoLoading /> : null}
      {!query.isLoading && !queryError && todos.length === 0 ? <TodoEmpty state={state} /> : null}
      {!query.isLoading && !queryError && todos.length > 0 ? (
        <div className="grid gap-3">
          {todos.map((todo) => (
            <TodoCard
              key={todo.id}
              todo={todo}
              isMarkingDone={markTodo.isPending && markTodo.activeTodoId === todo.id}
              onMarkDone={() => void markTodo.markDone(todo.id)}
            />
          ))}
        </div>
      ) : null}
    </div>
  );
}

function TodoCard({
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
    <Card className="card-enter overflow-hidden">
      <CardHeader className="gap-3 pb-3 sm:flex-row sm:items-start sm:justify-between sm:space-y-0">
        <div className="flex min-w-0 gap-3">
          <span className="flex size-10 shrink-0 items-center justify-center rounded-xl bg-primary/10 text-primary">
            {todo.state === "done" ? <CircleCheckBig className="size-5" /> : <Inbox className="size-5" />}
          </span>
          <div className="min-w-0">
            <CardTitle className="leading-5">{todo.title}</CardTitle>
            <p className="mt-2 text-sm leading-6 text-muted-foreground">{todo.summary}</p>
          </div>
        </div>
        <Badge variant={todo.state === "done" ? "secondary" : "default"}>{t(todo.state)}</Badge>
      </CardHeader>
      <CardContent className="flex flex-wrap items-center justify-between gap-3 pt-0">
        <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
          <Badge variant="outline">{formatKind(todo.kind)}</Badge>
          <span>{dayjs(todo.created_at).format("MMM D, YYYY h:mm A")}</span>
        </div>
        <div className="flex items-center gap-2">
          <Button asChild variant="outline" size="sm">
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
      </CardContent>
    </Card>
  );
}

function TodoLoading(): JSX.Element {
  return (
    <div className="grid gap-3" aria-label="Loading todos">
      {[0, 1, 2].map((item) => <Skeleton key={item} className="h-32 w-full rounded-2xl" />)}
    </div>
  );
}

function TodoEmpty({ state }: { state: TodoState }): JSX.Element {
  const { t } = useI18n();
  return (
    <Card className="border-dashed">
      <CardContent className="flex min-h-52 flex-col items-center justify-center gap-3 text-center">
        <span className="flex size-12 items-center justify-center rounded-full bg-primary/10 text-primary">
          <CircleCheckBig className="size-6" />
        </span>
        <p className="font-semibold">{t(state === "pending" ? "You're all caught up." : "No completed todos yet.")}</p>
        <p className="max-w-md text-sm text-muted-foreground">
          {t(state === "pending" ? "New project activity will appear here." : "Completed items remain available for reference.")}
        </p>
      </CardContent>
    </Card>
  );
}

const formatKind = (kind: string): string => kind.split("_").map((part) => part[0]?.toUpperCase() + part.slice(1)).join(" ");

const resolveActionUrl = (todo: TodoView): string =>
  todo.action_url.startsWith("/app/")
    ? todo.action_url
    : `/app/projects/${todo.organization_id}/${todo.project_id}`;
