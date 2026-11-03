import dayjs from "dayjs";
import { ArrowUpRight, CircleCheckBig, GitPullRequest, Inbox } from "lucide-react";
import { Link } from "react-router";
import { useI18n } from "@/lib/i18n";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { DataList, DataRow } from "@/components/surface/data-list";
import type { RepositoryView } from "@/pages/types";
import type { TodoKind, TodoView } from "../todo-types";

interface WorkListProps {
  items: TodoView[];
  repos: RepositoryView[];
  selectedId: string | null;
  onSelect: (id: string) => void;
}

export const WorkList = ({ items, repos, selectedId, onSelect }: WorkListProps): JSX.Element => {
  const { t } = useI18n();
  if (!items.length) return <div className="flex min-h-52 flex-col items-center justify-center gap-2 px-6 py-8 text-center">
    <CircleCheckBig className="size-7 text-primary" aria-hidden="true" />
    <p className="text-sm font-medium">{t("No pending work in this view.")}</p>
    <p className="text-sm text-muted-foreground">{t("New project activity assigned to you will appear here.")}</p>
  </div>;
  return <DataList>{items.slice(0, 8).map((todo) => {
    const repo = repos.find((item) => item.id === todo.project_id);
    const active = selectedId === todo.id;
    return <DataRow key={todo.id} onClick={() => onSelect(todo.id)} className={active ? "bg-primary/5 ring-1 ring-inset ring-primary/40" : ""}>
      <span className="flex size-8 shrink-0 items-center justify-center rounded-md bg-primary/10 text-primary"><GitPullRequest className="size-4" aria-hidden="true" /></span>
      <span className="min-w-0 flex-1"><span className="flex flex-wrap items-center gap-2"><span className="truncate text-sm font-medium">{todo.title}</span><Badge variant="outline">{todoKindLabel(todo.kind, t)}</Badge></span><span className="mt-1 block truncate text-xs text-muted-foreground">{repo?.name ?? todo.summary} · {dayjs(todo.created_at).format("MMM D, HH:mm")}</span></span>
      <span className="hidden text-xs text-muted-foreground sm:block">{todo.summary}</span>
    </DataRow>;
  })}</DataList>;
};

export const WorkPreview = ({ todo, repo }: { todo: TodoView | null; repo: RepositoryView | null }): JSX.Element => {
  const { t } = useI18n();
  if (!todo) return <aside className="flex min-h-64 flex-col items-center justify-center rounded-md border border-border bg-card p-6 text-center">
    <Inbox className="size-6 text-muted-foreground" aria-hidden="true" /><p className="mt-3 text-sm font-medium">{t("Select an item to see details")}</p>
    <p className="mt-1 text-sm text-muted-foreground">{t("Choose a pending item from your work list.")}</p>
  </aside>;
  const href = todo.action_url.startsWith("/app/") ? todo.action_url : `/app/projects/${todo.organization_id}/${todo.project_id}`;
  return <aside className="h-fit rounded-md border border-border bg-card" aria-label={t("Work details")}>
    <div className="border-b border-border p-5"><div className="flex items-start gap-3"><span className="flex size-9 shrink-0 items-center justify-center rounded-md bg-primary/10 text-primary"><GitPullRequest className="size-5" aria-hidden="true" /></span><div className="min-w-0"><Badge variant="secondary">{todoKindLabel(todo.kind, t)}</Badge><h2 className="mt-2 text-base font-semibold">{todo.title}</h2></div></div>
      <p className="mt-3 text-sm text-muted-foreground">{todo.summary}</p>
    </div>
    <div className="space-y-4 p-5"><Detail label={t("Project")} value={repo?.name ?? t("Project unavailable")} /><Detail label={t("Created")} value={dayjs(todo.created_at).format("MMM D, YYYY HH:mm")} />
      {repo ? <Detail label={t("Default branch")} value={repo.default_branch} /> : null}
      <div className="flex flex-col gap-2 border-t border-border pt-4"><Button asChild><Link to={href}>{t("Open item")}<ArrowUpRight aria-hidden="true" /></Link></Button><Button asChild variant="outline"><Link to={`/app/projects/${todo.organization_id}/${todo.project_id}`}>{t("Open project")}</Link></Button></div>
    </div>
  </aside>;
};

const Detail = ({ label, value }: { label: string; value: string }): JSX.Element => {
  return <div className="flex items-start justify-between gap-4 text-sm"><span className="text-muted-foreground">{label}</span><span className="text-right font-medium">{value}</span></div>;
};

const todoKindLabel = (kind: TodoKind, t: (key: string) => string): string => {
  const labels: Record<TodoKind, string> = {
    project_created: "Project created",
    issue_created: "Issue created",
    issue_assigned: "Issue assigned",
    issue_commented: "Issue commented",
    branch_protection_changed: "Branch protection changed",
    branch_deleted: "Branch deleted",
    merge_request_merged: "Merge request merged",
    merge_request_commented: "Merge request commented",
    merge_request_approved: "Merge request approved",
    merge_request_unapproved: "Merge request approval removed",
    merge_request_review_requested: "Merge request review requested",
    merge_request_assigned: "Merge request assigned",
  };
  return t(labels[kind]);
};
